use axum::{extract::{Path, State}, Json};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::{ApiResponse, UpsertMenuRequest};
use crate::modules::sys::service::menu_service as rbac_service;
use crate::AppState;

// ============== 菜单管理 =============

pub async fn list_menus(State(state): State<AppState>) -> Json<serde_json::Value> {
    match rbac_service::list_menus(&state.db.pool()).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "创建菜单", capture = req)]
pub async fn create_menu(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<UpsertMenuRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_menu(
        &state.db.pool(),
        &req.name,
        &req.title,
        req.parent_id,
        req.icon.as_deref(),
        req.path.as_deref(),
        req.component.as_deref(),
        req.menu_type,
        req.permission.as_deref(),
        req.sort.unwrap_or(0),
    )
    .await {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "更新菜单", capture = req)]
pub async fn update_menu(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertMenuRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_menu(
        &state.db.pool(),
        id,
        Some(req.name.as_str()),
        Some(req.title.as_str()),
        Some(req.parent_id),
        Some(req.icon.as_deref()),
        Some(req.path.as_deref()),
        Some(req.component.as_deref()),
        Some(req.permission.as_deref()),
        req.sort,
    )
    .await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "删除菜单")]
pub async fn delete_menu(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_menu(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("菜单不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "批量删除菜单", capture = ids)]
pub async fn batch_delete_menus(
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(ids): Json<Vec<i64>>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_menus(&state.db.pool(), &ids).await {
        Ok(n) => Json(json!(ApiResponse::success(format!("已删除 {} 条", n)))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}
