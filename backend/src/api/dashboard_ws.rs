use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;

use crate::service::token_service;
use crate::AppState;

#[derive(Deserialize)]
pub struct DashboardQuery {
    token: String,
}

pub async fn dashboard_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
) -> impl IntoResponse {
    if token_service::verify_token(state.db.pool(), &query.token).await.is_err() {
        return axum::http::Response::builder()
            .status(401)
            .body("Unauthorized".into())
            .unwrap();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let mut rx = state.dashboard_tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    // 立即推送一次当前数据
    let data = collect_dashboard_data(&state).await;
    let _ = sender.send(Message::Text(data.into())).await;

    // 转发广播消息到 WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(data) = rx.recv().await {
            if sender.send(Message::Text(data.into())).await.is_err() {
                break;
            }
        }
    });

    // 读取 WebSocket（只处理 close）
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

/// 启动后台推送任务（在 main.rs 中调用）
pub fn start_push_task(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        interval.tick().await; // 跳过第一次立即触发
        loop {
            interval.tick().await;
            let data = collect_dashboard_data(&state).await;
            // 没有订阅者时 send 会返回 Err，忽略即可
            let _ = state.dashboard_tx.send(data);
        }
    });
}

/// 操作 Nginx 后手动触发立即推送
pub async fn trigger_push(state: &AppState) {
    let data = collect_dashboard_data(state).await;
    let _ = state.dashboard_tx.send(data);
}

async fn collect_dashboard_data(state: &AppState) -> String {
    use crate::service::dashboard_service;
    use crate::nginx;

    let config = state.get_config();
    let nginx_bin = config.nginx.bin.clone();

    // 并发采集 dashboard 统计和 nginx 状态
    let (dashboard, nginx_status) = tokio::join!(
        dashboard_service::get_dashboard(state),
        nginx::get_nginx_status(&nginx_bin),
    );

    let stats = dashboard.unwrap_or_else(|_| crate::dto::DashboardData {
        nginx_version: String::new(),
        worker_count: 0,
        active_connections: 0,
        site_count: 0,
        cert_count: 0,
        cpu_usage: 0.0,
        memory_usage: 0.0,
        memory_total: 0,
        app_memory: 0,
    });

    serde_json::json!({
        "nginx": {
            "running": nginx_status.running,
            "pid": nginx_status.pid,
            "version": nginx_status.version,
            "uptime": nginx_status.uptime,
            "not_installed": nginx_status.not_installed,
        },
        "stats": {
            "site_count": stats.site_count,
            "cert_count": stats.cert_count,
            "cpu_usage": stats.cpu_usage,
            "memory_usage": stats.memory_usage,
            "memory_total": stats.memory_total,
            "app_memory": stats.app_memory,
        }
    })
    .to_string()
}
