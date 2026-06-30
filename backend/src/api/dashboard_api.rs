use axum::{extract::State, Json};
use serde_json::json;
use tracing::error;
// use tracing::{debug, error, info};

use crate::dto::ApiResponse;
use crate::service::dashboard_service;
use crate::AppState;

/// 获取Dashboard数据
pub async fn get_dashboard(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // info!("[Dashboard] 获取Dashboard数据请求");
    // debug!("[Dashboard] state config: server={}:{}", state.config.server.host, state.config.server.port);
    match dashboard_service::get_dashboard(&state).await {
        Ok(data) => {
            // debug!(
            //     "[Dashboard] 成功: nginx_version={}, site_count={}, cert_count={}, cpu_usage={:.1}%, memory_usage={:.1}%",
            //     data.nginx_version, data.site_count, data.cert_count, data.cpu_usage, data.memory_usage
            // );
            Json(json!(ApiResponse::success(data)))
        }
        Err(e) => {
            error!("[Dashboard] 获取Dashboard数据失败: {}", e);
            Json(json!(ApiResponse::<()>::error(format!("获取Dashboard数据失败: {}", e))))
        }
    }
}
