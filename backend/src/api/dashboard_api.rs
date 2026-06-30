use axum::{extract::State, Json};
use serde_json::json;

use crate::dto::ApiResponse;
use crate::service::dashboard_service;
use crate::AppState;

/// 获取Dashboard数据
pub async fn get_dashboard(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match dashboard_service::get_dashboard(&state).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取Dashboard数据失败: {}", e)))),
    }
}
