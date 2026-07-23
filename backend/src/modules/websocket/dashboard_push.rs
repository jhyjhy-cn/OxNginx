use crate::AppState;

/// 启动后台 dashboard 推送任务（在 main.rs 中调用）
/// ponytail: 从旧 modules/dashboard/controller/dashboard_ws.rs 迁来
pub fn start_dashboard_push_task(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        interval.tick().await; // 跳过第一次立即触发
        loop {
            interval.tick().await;
            let data = collect_dashboard_data(&state).await;
            publish(&state, data);
        }
    });
}

/// 操作 Nginx 后手动触发立即推送
pub async fn trigger_dashboard_push(state: &AppState) {
    let data = collect_dashboard_data(state).await;
    publish(state, data);
}

/// 写缓存 + 广播:后台任务与手动触发共用。
/// 缓存供客户端订阅时即时回传,广播负责把新数据推给已订阅的连接
fn publish(state: &AppState, data: String) {
    if let Ok(v) = serde_json::from_str(&data) {
        *state.dashboard_cache.write() = v;
    }
    // 没有订阅者时 send 会返回 Err，忽略即可
    let _ = state.dashboard_tx.send(data);
}

/// ponytail: 同步立即取一次快照，hub 处理 subscribe 时调用
pub async fn collect_dashboard_data_now(state: &AppState) -> serde_json::Value {
    let json_str = collect_dashboard_data(state).await;
    serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null)
}

async fn collect_dashboard_data(state: &AppState) -> String {
    use crate::modules::common::nginx;
    use crate::modules::dashboard::service::dashboard_service;

    let nginx_config = nginx::get_nginx_config(state).await.unwrap_or_default();
    let nginx_bin = nginx_config.bin.as_deref().unwrap_or("");

    let (dashboard, nginx_status) = tokio::join!(
        dashboard_service::get_dashboard(state),
        nginx::get_nginx_status(nginx_bin),
    );

    let stats = dashboard.unwrap_or_else(|_| crate::modules::common::dto::DashboardData {
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