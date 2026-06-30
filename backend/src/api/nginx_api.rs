use axum::{extract::State, Json};
use serde_json::json;

use crate::dto::{ApiResponse, NginxTestResult};
use crate::AppState;

/// 测试Nginx配置
pub async fn test_config(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let result = crate::nginx::test_config(&state.config.nginx.bin).await;
    Json(json!(ApiResponse::success(result)))
}

/// 重载Nginx配置
pub async fn reload(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // 先测试配置
    let test_result = crate::nginx::test_config(&state.config.nginx.bin).await;
    if !test_result.success {
        return Json(json!(ApiResponse::<()>::error(format!(
            "配置测试失败，禁止重载: {}",
            test_result.message
        ))));
    }

    // 重载
    match crate::nginx::reload_nginx(&state.config.nginx.bin).await {
        Ok(true) => Json(json!(ApiResponse::success(NginxTestResult {
            success: true,
            message: "Nginx重载成功".into(),
        }))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx重载失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx重载失败: {}", e)))),
    }
}
