use axum::{extract::{Extension, Path, State}, Json};
use serde_json::json;

use ox_nginx_macros::{audit_log, check_permission};

use crate::modules::common::dto::{ApiResponse, UpsertDictRequest, UpsertDictItemRequest};
use crate::modules::common::middleware::TokenInfo;
use crate::modules::sys::service::dict_service as rbac_service;
use crate::AppState;

// ============== 字典管理 =============

#[check_permission("sys:dict:query")]
pub async fn list_dicts(State(state): State<AppState>, token: Extension<TokenInfo>) -> Json<serde_json::Value> {
    match rbac_service::list_dicts(&state.db.pool()).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dict:query")]
pub async fn get_dict(State(state): State<AppState>, Path(id): Path<i64>, token: Extension<TokenInfo>) -> Json<serde_json::Value> {
    match rbac_service::get_dict_with_items(&state.db.pool(), id).await {
        Ok(Some(data)) => Json(json!(ApiResponse::success(data))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("字典不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dict:add")]
#[audit_log(module = "rbac", action = "创建字典", capture = req)]
pub async fn create_dict(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<UpsertDictRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_dict(&state.db.pool(), &req.name, &req.code, req.remark.as_deref()).await {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dict:edit")]
#[audit_log(module = "rbac", action = "更新字典", capture = req)]
pub async fn update_dict(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertDictRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_dict(&state.db.pool(), id, Some(&req.name), req.remark.as_deref(), None).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dict:delete")]
#[audit_log(module = "rbac", action = "删除字典")]
pub async fn delete_dict(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>, Path(id): Path<i64>) -> Json<serde_json::Value> {
    match rbac_service::delete_dict(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("字典不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dict:dictItemManage")]
#[audit_log(module = "rbac", action = "创建字典项", capture = req)]
pub async fn create_dict_item(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(dict_id): Path<i64>,
    Json(req): Json<UpsertDictItemRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_dict_item(&state.db.pool(), dict_id, &req.label, &req.value, req.sort.unwrap_or(0)).await {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dict:dictItemManage")]
#[audit_log(module = "rbac", action = "更新字典项", capture = req)]
pub async fn update_dict_item(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertDictItemRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_dict_item(&state.db.pool(), id, Some(&req.label), Some(&req.value), req.sort, req.status).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:dict:dictItemManage")]
#[audit_log(module = "rbac", action = "删除字典项")]
pub async fn delete_dict_item(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>, Path(id): Path<i64>) -> Json<serde_json::Value> {
    match rbac_service::delete_dict_item(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("字典项不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}
