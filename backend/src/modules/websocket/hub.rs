use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

use crate::AppState;
use super::dashboard_push;
use super::protocol::{cmd as Cmd, Channel, ClientFrame, ServerEvent, ServerFrame};

/// 单条连接的运行时状态
#[derive(Default)]
pub struct ConnState {
    /// 客户端声明订阅的通道集合
    pub channels: HashSet<Channel>,
}

/// ponytail: parking_lot::Mutex 无 poison、无 unwrap；conn 仅在 select 分支里读 snapshot
type SharedConn = Arc<parking_lot::Mutex<ConnState>>;

/// 解析客户端文本帧为结构化指令
fn parse_client(text: &str) -> Option<ClientFrame> {
    serde_json::from_str::<ClientFrame>(text).ok()
}

/// 把 ServerFrame 序列化为文本
fn frame_to_text(f: &ServerFrame<'_>) -> String {
    serde_json::to_string(f).unwrap_or_else(|_| format!(r#"{{"cmd":"{}"}}"#, Cmd::PONG))
}

fn event_frame(ev: &ServerEvent) -> String {
    frame_to_text(&ServerFrame::Event(ev))
}

/// 单入口 ws 处理器（dashboard + events 共用一条连接）
pub async fn serve(socket: WebSocket, state: AppState, token_id: i64) {
    let (mut sender, mut receiver) = socket.split();
    let conn: SharedConn = Arc::new(parking_lot::Mutex::new(ConnState::default()));

    // ponytail: 一条连接两个上游 channel（dashboard + events），各订阅一份
    let mut dashboard_rx = state.dashboard_tx.subscribe();
    let mut event_rx = state.event_tx.subscribe();

    let mut heartbeat = tokio::time::interval(Duration::from_secs(30));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last_recv = Instant::now();
    let max_idle = Duration::from_secs(90);

    loop {
        // ponytail: 调试日志（trace 链路用）
        // tracing::debug!("[ws-hub] loop tick token_id={}", token_id);
        tokio::select! {
            biased;
            maybe_msg = receiver.next() => {
                // tracing::debug!("[ws-hub] receiver.next = {:?}", maybe_msg.as_ref().map(|r| match r {
                //     Ok(m) => format!("{:?}", m),
                //     Err(e) => format!("Err({:?})", e),
                // }));
                match maybe_msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(p))) => {
                        if sender.send(Message::Pong(p)).await.is_err() { break; }
                        last_recv = Instant::now();
                    }
                    Some(Ok(Message::Pong(_))) => { last_recv = Instant::now(); }
                    Some(Ok(Message::Text(text))) => {
                        // tracing::debug!("[ws-hub] TEXT frame: {}", text);
                        last_recv = Instant::now();
                        handle_client_text(&text, &state, &conn, &mut sender).await;
                    }
                    Some(Ok(Message::Binary(_))) => { last_recv = Instant::now(); }
                    Some(Err(_)) => break,
                }
            }
            data = dashboard_rx.recv() => {
                let channels = conn.lock().channels.clone();
                // tracing::debug!(target: "ws", "dashboard_rx recv channels={:?}", channels);
                if !channels.contains(&Channel::Dashboard) { continue; }
                if push_dashboard_payload(&data, &mut sender).await.is_err() { break; }
            }
            ev = event_rx.recv() => {
                let channels = conn.lock().channels.clone();
                if !channels.contains(&Channel::Events) { continue; }
                match ev {
                    Ok(ServerEvent::Kick { token_id: kicked, .. }) => {
                        if kicked == token_id {
                            let s = event_frame(&ServerEvent::Kick { token_id: kicked, reason: 0 });
                            if sender.send(Message::Text(s.into())).await.is_err() { break; }
                        }
                    }
                    Ok(other) => {
                        let s = event_frame(&other);
                        if sender.send(Message::Text(s.into())).await.is_err() { break; }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
            _ = heartbeat.tick() => {
                if last_recv.elapsed() > max_idle { break; }
                if sender.send(Message::Ping(axum::body::Bytes::from_static(b"hb"))).await.is_err() { break; }
            }
        }
    }

    let _ = sender.close().await;
}

async fn push_dashboard_payload(
    data: &Result<String, tokio::sync::broadcast::error::RecvError>,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Result<(), ()> {
    let Ok(json_str) = data else { return Ok(()); };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) else { return Ok(()); };
    let s = frame_to_text(&ServerFrame::Dashboard(&v));
    sender.send(Message::Text(s.into())).await.map_err(|_| ())
}

async fn handle_client_text(
    text: &str,
    state: &AppState,
    conn: &SharedConn,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) {
    // ponytail: 调试日志（trace 链路用）
    // tracing::debug!("[ws-hub] handle_client_text: {}", text);
    let Some(frame) = parse_client(text) else {
        tracing::warn!("[ws] parse_client failed: {}", text);
        return;
    };
    // tracing::debug!("[ws-hub] parsed cmd={}", frame.cmd());
    match frame {
        ClientFrame::Ping {} => {
            let pong = serde_json::json!({ "cmd": Cmd::PONG }).to_string();
            let _ = sender.send(Message::Text(pong.into())).await;
        }
        ClientFrame::Subscribe { channels, .. } => {
            let mut wants_dashboard = false;
            {
                let mut g = conn.lock();
                for c in &channels {
                    if let Some(ch) = Channel::parse(c) {
                        if ch == Channel::Dashboard {
                            wants_dashboard = true;
                        }
                        g.channels.insert(ch);
                    }
                }
            }
            // 订阅 dashboard:立即回最近缓存快照(后台 10s 刷新、nginx 操作后即时刷新),
            // 不再现场采集(wmic/磁盘查询要数秒)。修复原先"每连接只推一次"导致
            // 页面切换后重新订阅要干等 10s tick 的问题
            if wants_dashboard {
                let mut snap = state.dashboard_cache.read().clone();
                if snap.is_null() {
                    // 少见:启动后 10s 内缓存还没生成,兜底同步采集一次
                    snap = dashboard_push::collect_dashboard_data_now(state).await;
                }
                let s = frame_to_text(&ServerFrame::Dashboard(&snap));
                let _ = sender.send(Message::Text(s.into())).await;
            }
        }
        ClientFrame::Unsubscribe { channels, .. } => {
            let mut g = conn.lock();
            for c in channels {
                if let Some(ch) = Channel::parse(&c) {
                    g.channels.remove(&ch);
                }
            }
        }
        _ => { /* terminal.* 帧在独立连接里处理 */ }
    }
}