use axum::{extract::State, Json};
use serde_json::json;

use crate::dto::{ApiResponse, LogResponse};
use crate::util::read_log_tail;
use crate::AppState;

/// 获取Access日志
pub async fn access_log(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    let log_path = "/var/log/nginx/access.log";
    match read_log_tail(log_path, 100).await {
        Ok(lines) => Json(json!(ApiResponse::success(LogResponse { lines }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取日志失败: {}", e)))),
    }
}

/// 获取Error日志
pub async fn error_log(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    let log_path = "/var/log/nginx/error.log";
    match read_log_tail(log_path, 100).await {
        Ok(lines) => Json(json!(ApiResponse::success(LogResponse { lines }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取日志失败: {}", e)))),
    }
}
