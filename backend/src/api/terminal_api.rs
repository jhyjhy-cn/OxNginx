use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::Deserialize;

use crate::auth;
use crate::AppState;

#[derive(Deserialize)]
pub struct TerminalQuery {
    token: String,
    cols: Option<u16>,
    rows: Option<u16>,
}

pub async fn terminal_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<TerminalQuery>,
) -> impl IntoResponse {
    let config = state.get_config();
    if auth::verify_token(&query.token, &config.auth.jwt_secret).is_err() {
        return axum::http::Response::builder()
            .status(401)
            .body("Unauthorized".into())
            .unwrap();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, query.cols, query.rows))
}

async fn handle_socket(socket: WebSocket, cols: Option<u16>, rows: Option<u16>) {
    let cols = cols.unwrap_or(80);
    let rows = rows.unwrap_or(24);
    tracing::info!("终端 WebSocket 已连接, cols={}, rows={}", cols, rows);

    let pty_system = NativePtySystem::default();
    let pty = match pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(pty) => pty,
        Err(e) => {
            tracing::error!("创建 PTY 失败: {}", e);
            return;
        }
    };

    #[cfg(target_os = "windows")]
    let mut cmd = CommandBuilder::new("powershell.exe");
    #[cfg(not(target_os = "windows"))]
    let mut cmd = CommandBuilder::new("/bin/sh");

    cmd.env("TERM", "xterm-256color");

    let mut child = match pty.slave.spawn_command(cmd) {
        Ok(child) => child,
        Err(e) => {
            tracing::error!("启动终端进程失败: {}", e);
            return;
        }
    };
    tracing::info!("终端进程已启动, pid={:?}", child.process_id());

    let reader = pty.master.try_clone_reader().unwrap();
    let writer = pty.master.take_writer().unwrap();

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // PTY reader → WebSocket（阻塞读，用 spawn_blocking 桥接到 tokio）
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
    tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut reader = reader;
        let mut buf = vec![0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    tracing::info!("PTY reader EOF");
                    break;
                }
                Ok(n) => {
                    tracing::debug!("PTY 读取 {} 字节", n);
                    if tx.blocking_send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("PTY 读取错误: {}", e);
                    break;
                }
            }
        }
    });

    // 转发 PTY 输出到 WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            tracing::debug!("转发 {} 字节到 WebSocket", data.len());
            if ws_sender.send(Message::Binary(data)).await.is_err() {
                tracing::warn!("WebSocket 发送失败");
                break;
            }
        }
        tracing::info!("PTY→WS 转发结束");
    });

    // WebSocket → PTY writer（处理输入和 resize）
    let writer = tokio::sync::Mutex::new(writer);
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if text.starts_with('\x01') {
                        if let Some(json_str) = text.get(1..) {
                            if let Ok(size) = serde_json::from_str::<serde_json::Value>(json_str)
                            {
                                let new_cols = size["cols"].as_u64().unwrap_or(80) as u16;
                                let new_rows = size["rows"].as_u64().unwrap_or(24) as u16;
                                tracing::info!("终端 resize: {}x{}", new_cols, new_rows);
                                let _ = pty.master.resize(PtySize {
                                    rows: new_rows,
                                    cols: new_cols,
                                    pixel_width: 0,
                                    pixel_height: 0,
                                });
                            }
                        }
                    } else {
                        tracing::debug!("WS 收到输入: {} 字节", text.len());
                        let mut writer = writer.lock().await;
                        use std::io::Write;
                        let _ = writer.write_all(text.as_bytes());
                        let _ = writer.flush();
                    }
                }
                Message::Binary(data) => {
                    tracing::debug!("WS 收到二进制输入: {} 字节", data.len());
                    let mut writer = writer.lock().await;
                    use std::io::Write;
                    let _ = writer.write_all(&data);
                    let _ = writer.flush();
                }
                Message::Close(_) => {
                    tracing::info!("终端 WebSocket 关闭");
                    break;
                }
                _ => {}
            }
        }
        tracing::info!("WS→PTY 转发结束");
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    let _ = child.kill();
    let _ = child.wait();
}
