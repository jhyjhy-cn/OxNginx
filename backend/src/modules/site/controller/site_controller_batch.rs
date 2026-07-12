use axum::Extension;
use crate::modules::common::audit::context::SharedAuditContext;
use axum::{
    extract::State,
    Json,
};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::{ApiResponse, UpdateSiteRequest};
use crate::modules::common::nginx::get_nginx_config;
use crate::modules::site::service::site_service;
use crate::AppState;

use super::site_controller::BatchRequest;

#[audit_log(module = "site", action = "批量启用站点", capture = req)]
pub async fn batch_enable(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Json(req): Json<BatchRequest>,
) -> Json<serde_json::Value> {
    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    let sites_enabled = nginx_config.sites_enabled.as_deref().unwrap_or("");
    let nginx_bin = nginx_config.bin.as_deref().unwrap_or("");

    let mut success_count = 0;
    let mut error_count = 0;

    for id in &req.ids {
        let update_req = UpdateSiteRequest {
            name: None,
            server_name: None,
            listen: None,
            ssl: None,
            certificate_path: None,
            key_path: None,
            proxy_pass: None,
            root_path: None,
            remark: None,
            expire_time: None,
            rewrite_rules: None,
            redirect_rules: None,
            hotlink_config: None,
            log_access_path: None,
            log_error_path: None,
            status: Some(1),
        };

        match site_service::update_site(&state, *id, update_req).await {
            Ok(Some(site)) => {
                let config_content = crate::modules::common::nginx::generate_site_config(&site);
                let _ = crate::modules::common::nginx::write_site_config(sites_enabled, &site.name, &config_content).await;
                success_count += 1;
            }
            _ => error_count += 1,
        }
    }

    // 重载 nginx 配置
    if !nginx_bin.is_empty() {
        let _ = crate::modules::common::nginx::reload_nginx(nginx_bin).await;
    }

    Json(json!(ApiResponse::success(serde_json::json!({
        "success": success_count,
        "error": error_count,
    }))))
}

/// 批量禁用站点
#[audit_log(module = "site", action = "批量禁用站点", capture = req)]
pub async fn batch_disable(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Json(req): Json<BatchRequest>,
) -> Json<serde_json::Value> {
    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    let sites_enabled = nginx_config.sites_enabled.as_deref().unwrap_or("");
    let nginx_bin = nginx_config.bin.as_deref().unwrap_or("");

    let mut success_count = 0;
    let mut error_count = 0;

    for id in &req.ids {
        let update_req = UpdateSiteRequest {
            name: None,
            server_name: None,
            listen: None,
            ssl: None,
            certificate_path: None,
            key_path: None,
            proxy_pass: None,
            root_path: None,
            remark: None,
            expire_time: None,
            rewrite_rules: None,
            redirect_rules: None,
            hotlink_config: None,
            log_access_path: None,
            log_error_path: None,
            status: Some(0),
        };

        match site_service::update_site(&state, *id, update_req).await {
            Ok(Some(site)) => {
                let _ = crate::modules::common::nginx::remove_site_config(sites_enabled, &site.name).await;
                success_count += 1;
            }
            _ => error_count += 1,
        }
    }

    // 重载 nginx 配置
    if !nginx_bin.is_empty() {
        let _ = crate::modules::common::nginx::reload_nginx(nginx_bin).await;
    }

    Json(json!(ApiResponse::success(serde_json::json!({
        "success": success_count,
        "error": error_count,
    }))))
}
