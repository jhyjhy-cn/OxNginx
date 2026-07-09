// 仅保留跨模块共享的接口：/api/rbac/me 和 /api/rbac/i18n/messages
// 其余按表拆分到 user_api.rs, role_api.rs 等

use axum::{
    extract::{Extension, State},
    Json,
};
use serde_json::json;

use crate::dto::{ApiResponse, RbacInfo};
use crate::middleware::TokenInfo;
use crate::service::rbac_service;
use crate::AppState;

/// 当前登录用户的 RBAC 信息
pub async fn me(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    match rbac_service::get_rbac_info(&state.db.pool(), &info.username).await {
        Ok((roles, permissions, menus)) => {
            Json(json!(ApiResponse::success(RbacInfo { roles, permissions, menus })))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

/// 获取 i18n 消息（任意登录用户可用）
pub async fn get_i18n_messages(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let locale = params.get("locale").map(|s| s.as_str()).unwrap_or("zh-CN");
    match rbac_service::get_i18n_messages(&state.db.pool(), locale).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}
