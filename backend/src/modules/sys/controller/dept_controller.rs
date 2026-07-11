use axum::{extract::{Extension, Path, State}, Json};
use serde_json::json;

use ox_nginx_macros::{audit_log, check_permission};

use crate::modules::common::dto::{ApiResponse, PagedResult, PageQuery, UpsertDeptRequest};
use crate::modules::common::middleware::TokenInfo;
use crate::modules::sys::service::dept_service as rbac_service;
use crate::AppState;

// ============== 部门管理 =============

#[check_permission(value = "sys:dept:query")]
pub async fn list_depts(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<PageQuery>,
    token: Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(100).max(1);
    match rbac_service::list_depts_paged(&state.db.pool(), page, page_size, q.keyword.as_deref()).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

/// 部门树（给前端左侧树用）
#[check_permission("sys:dept:query")]
pub async fn dept_tree(
    State(state): State<AppState>,
    token: Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    match rbac_service::list_dept_tree(&state.db.pool()).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dept:add")]
#[audit_log(module = "rbac", action = "创建部门", capture = req)]
pub async fn create_dept(
    State(state): State<AppState>,
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    Json(req): Json<UpsertDeptRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_dept(&state.db.pool(), &req.name, req.parent_id, req.sort.unwrap_or(0)).await {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dept:update")]
#[audit_log(module = "rbac", action = "更新部门", capture = req)]
pub async fn update_dept(
    State(state): State<AppState>,
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertDeptRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_dept(&state.db.pool(), id, Some(req.name.as_str()), Some(req.parent_id), req.sort).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dept:delete")]
#[audit_log(module = "rbac", action = "删除部门")]
pub async fn delete_dept(
    State(state): State<AppState>,
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_dept(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("部门不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dept:batch-delete")]
#[audit_log(module = "rbac", action = "批量删除部门", capture = ids)]
pub async fn batch_delete_depts(
    State(state): State<AppState>,
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    Json(ids): Json<Vec<i64>>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_depts(&state.db.pool(), &ids).await {
        Ok(n) => Json(json!(ApiResponse::success(format!("已删除 {} 条", n)))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}
