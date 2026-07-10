use axum::{extract::{Path, State}, Json};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::{ApiResponse, PagedResult, PageQuery, SetRoleMenusRequest, UpsertRoleRequest};
use crate::modules::sys::service::role_service as rbac_service;
use crate::AppState;

// ============== 角色管理 =============

pub async fn list_roles(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<PageQuery>,
) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);
    match rbac_service::list_roles_paged(&state.db.pool(), page, page_size, q.keyword.as_deref()).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "创建角色", capture = req)]
pub async fn create_role(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<UpsertRoleRequest>,
) -> Json<serde_json::Value> {
    let r = rbac_service::create_role(&state.db.pool(), &req.code, &req.name, req.remark.as_deref()).await;
    match r {
        Ok(id) => {
            if let Some(mids) = req.menu_ids {
                let _ = rbac_service::set_role_menus(&state.db.pool(), id, &mids).await;
            }
            Json(json!(ApiResponse::success(id)))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "更新角色", capture = req)]
pub async fn update_role(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertRoleRequest>,
) -> Json<serde_json::Value> {
    let r = rbac_service::update_role(&state.db.pool(), id, Some(req.name.as_str()), req.remark.as_deref(), req.status).await;
    match r {
        Ok(_) => {
            if let Some(mids) = req.menu_ids {
                let _ = rbac_service::set_role_menus(&state.db.pool(), id, &mids).await;
            }
            Json(json!(ApiResponse::success("ok")))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "删除角色")]
pub async fn delete_role(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_role(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("角色不存在或不可删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "设置角色菜单", capture = req)]
pub async fn set_role_menus(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<SetRoleMenusRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::set_role_menus(&state.db.pool(), id, &req.menu_ids).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn get_role_menus(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::get_role_menus(&state.db.pool(), id).await {
        Ok(ids) => Json(json!(ApiResponse::success(ids))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}
