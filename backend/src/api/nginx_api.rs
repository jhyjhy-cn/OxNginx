use axum::{extract::{Extension, State}, Json};
use serde_json::json;

use crate::dto::{ApiResponse, NginxTestResult};
use crate::middleware::TokenInfo;
use crate::AppState;

pub async fn test_config(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = state.get_config();
    let result = crate::nginx::test_config(&config.nginx.bin).await;
    Json(json!(ApiResponse::success(result)))
}

pub async fn reload(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let test_result = crate::nginx::test_config(&config.nginx.bin).await;
    if !test_result.success {
        return Json(json!(ApiResponse::<()>::error(format!("配置测试失败，禁止重载: {}", test_result.message))));
    }
    let result = match crate::nginx::reload_nginx(&config.nginx.bin).await {
        Ok(true) => Ok(NginxTestResult { success: true, message: "Nginx重载成功".into() }),
        Ok(false) => Err("Nginx重载失败".to_string()),
        Err(e) => Err(format!("Nginx重载失败: {}", e)),
    };
    crate::api::dashboard_ws::trigger_push(&state).await;
    if result.is_ok() { crate::service::log_service::log_op(state.db.pool(), &info.username, "重载Nginx配置").await; }
    match result {
        Ok(r) => Json(json!(ApiResponse::success(r))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e))),
    }
}

pub async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = state.get_config();
    Json(json!(ApiResponse::success(crate::nginx::get_nginx_status(&config.nginx.bin).await)))
}

pub async fn start(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    match crate::nginx::start_nginx(&config.nginx.bin, &config.nginx.config).await {
        Ok(true) => {
            crate::api::dashboard_ws::trigger_push(&state).await;
            crate::service::log_service::log_op(state.db.pool(), &info.username, "启动Nginx").await;
            Json(json!(ApiResponse::success("Nginx已启动")))
        }
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx启动失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx启动失败: {}", e)))),
    }
}

pub async fn stop(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    match crate::nginx::stop_nginx(&config.nginx.bin).await {
        Ok(true) => {
            crate::api::dashboard_ws::trigger_push(&state).await;
            crate::service::log_service::log_op(state.db.pool(), &info.username, "停止Nginx").await;
            Json(json!(ApiResponse::success("Nginx已停止")))
        }
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx停止失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx停止失败: {}", e)))),
    }
}

pub async fn restart(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    match crate::nginx::restart_nginx(&config.nginx.bin, &config.nginx.config).await {
        Ok(true) => {
            crate::api::dashboard_ws::trigger_push(&state).await;
            crate::service::log_service::log_op(state.db.pool(), &info.username, "重启Nginx").await;
            Json(json!(ApiResponse::success("Nginx已重启")))
        }
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx重启失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx重启失败: {}", e)))),
    }
}

pub async fn install(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let install_dir = std::path::Path::new(&config.nginx.bin)
        .parent().and_then(|p| p.parent())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".into());
    match crate::nginx::install_nginx(&install_dir).await {
        Ok(_) => {
            crate::api::dashboard_ws::trigger_push(&state).await;
            crate::service::log_service::log_op(state.db.pool(), &info.username, "一键安装Nginx").await;
            Json(json!(ApiResponse::success("Nginx安装成功")))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("安装失败: {}", e)))),
    }
}
