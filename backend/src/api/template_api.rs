use axum::Extension;
use crate::audit::context::SharedAuditContext;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::dto::{ApiResponse, CreateTemplateRequest, UpdateTemplateRequest};
use crate::service::template_service;
use crate::AppState;

/// 获取配置模板列表
pub async fn list_templates(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match template_service::get_all_templates(&state).await {
        Ok(templates) => Json(json!(ApiResponse::success(templates))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取模板列表失败: {}", e)))),
    }
}

/// 获取单个配置模板
pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match template_service::get_template(&state, id).await {
        Ok(Some(template)) => Json(json!(ApiResponse::success(template))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("模板不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取模板失败: {}", e)))),
    }
}

/// 创建配置模板
#[audit_log(module = "template", action = "创建模板", capture = req)]
pub async fn create_template(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
) -> Json<serde_json::Value> {
    match template_service::create_template(&state, req).await {
        Ok(template) => Json(json!(ApiResponse::success(template))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建模板失败: {}", e)))),
    }
}

/// 更新配置模板
#[audit_log(module = "template", action = "更新模板", capture = req)]
pub async fn update_template(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTemplateRequest>,
) -> Json<serde_json::Value> {
    match template_service::update_template(&state, id, req).await {
        Ok(Some(template)) => Json(json!(ApiResponse::success(template))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("模板不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("更新模板失败: {}", e)))),
    }
}

/// 删除配置模板
#[audit_log(module = "template", action = "删除模板")]
pub async fn delete_template(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match template_service::delete_template(&state, id).await {
        Ok(true) => Json(json!(ApiResponse::success("模板已删除"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("删除模板失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除模板失败: {}", e)))),
    }
}

/// 预览模板应用效果
pub async fn preview_template(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(variables): Json<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    match template_service::get_template(&state, id).await {
        Ok(Some(template)) => {
            let config = template_service::apply_template_variables(&template.config, &variables);
            Json(json!(ApiResponse::success(serde_json::json!({
                "config": config,
            }))))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("模板不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取模板失败: {}", e)))),
    }
}
