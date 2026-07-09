use axum::Extension;
use crate::modules::common::audit::context::SharedAuditContext;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::{ApiResponse, CreateAccessRuleRequest, UpdateAccessRuleRequest};
use crate::modules::site::service::access_service;
use crate::AppState;

/// 获取访问控制规则列表
pub async fn list_rules(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match access_service::get_all_rules(&state).await {
        Ok(rules) => Json(json!(ApiResponse::success(rules))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取规则列表失败: {}", e)))),
    }
}

/// 获取单个访问控制规则
pub async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match access_service::get_rule(&state, id).await {
        Ok(Some(rule)) => Json(json!(ApiResponse::success(rule))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("规则不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取规则失败: {}", e)))),
    }
}

/// 创建访问控制规则
#[audit_log(module = "access", action = "创建访问规则", capture = req)]
pub async fn create_rule(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Json(req): Json<CreateAccessRuleRequest>,
) -> Json<serde_json::Value> {
    match access_service::create_rule(&state, req).await {
        Ok(rule) => Json(json!(ApiResponse::success(rule))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建规则失败: {}", e)))),
    }
}

/// 更新访问控制规则
#[audit_log(module = "access", action = "更新访问规则", capture = req)]
pub async fn update_rule(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAccessRuleRequest>,
) -> Json<serde_json::Value> {
    match access_service::update_rule(&state, id, req).await {
        Ok(Some(rule)) => Json(json!(ApiResponse::success(rule))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("规则不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("更新规则失败: {}", e)))),
    }
}

/// 删除访问控制规则
#[audit_log(module = "access", action = "删除访问规则")]
pub async fn delete_rule(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match access_service::delete_rule(&state, id).await {
        Ok(true) => Json(json!(ApiResponse::success("规则已删除"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("删除规则失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除规则失败: {}", e)))),
    }
}
