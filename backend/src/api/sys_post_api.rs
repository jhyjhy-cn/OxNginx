use axum::{extract::{Path, State}, Json};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::dto::{ApiResponse, PagedResult, PageQuery, UpsertPostRequest};
use crate::service::rbac_service;
use crate::AppState;

// ============== 岗位管理 =============

pub async fn list_posts(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<PageQuery>,
) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);
    match rbac_service::list_posts_paged(&state.db.pool(), page, page_size, q.keyword.as_deref()).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "创建岗位", capture = req)]
pub async fn create_post(
    ctx: axum::extract::Extension<crate::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<UpsertPostRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_post(&state.db.pool(), &req.code, &req.name, req.sort.unwrap_or(0)).await {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "更新岗位", capture = req)]
pub async fn update_post(
    ctx: axum::extract::Extension<crate::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertPostRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_post(&state.db.pool(), id, Some(req.name.as_str()), req.sort).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "删除岗位")]
pub async fn delete_post(
    ctx: axum::extract::Extension<crate::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_post(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("岗位不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}
