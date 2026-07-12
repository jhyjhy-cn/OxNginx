use axum::{extract::{Extension, Path, State}, Json};
use serde_json::json;

use ox_nginx_macros::{audit_log, check_permission};

use crate::modules::common::dto::{ApiResponse, PagedResult};
use crate::modules::common::middleware::TokenInfo;
use crate::modules::sys::service::online_service;
use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct OnlineQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(default, deserialize_with = "crate::modules::common::dto::empty_str_opt")]
    pub keyword: Option<String>,
}

#[check_permission("sys:online:view")]
pub async fn list_online(
    State(state): State<AppState>,
    Extension(token): Extension<TokenInfo>,
    axum::extract::Query(q): axum::extract::Query<OnlineQuery>,
) -> Json<serde_json::Value> {
    let _ = token;
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    match online_service::list_online(&state.db.pool(), page, page_size, q.keyword.as_deref()).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult {
            list,
            total,
            page,
            page_size,
        }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:online:kick")]
#[audit_log(module = "rbac", action = "强退会话")]
pub async fn kick_online(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Extension(token): Extension<TokenInfo>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    let _ = ctx;
    let info = token;
    // 禁止强退当前会话
    if id == info.token_id {
        return Json(json!(ApiResponse::<()>::error("不能强退当前会话")));
    }
    match online_service::kick_online(&state, id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("会话不存在或已过期"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}