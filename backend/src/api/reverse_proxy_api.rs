use axum::Extension;
use crate::audit::context::SharedAuditContext;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::dto::ApiResponse;
use crate::service::reverse_proxy_service;
use crate::AppState;
use ox_nginx_macros::audit_log;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateProxyRequest {
    pub name: String,
    pub proxy_dir: Option<String>,
    pub target_url: String,
    pub cache: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateProxyRequest {
    pub name: Option<String>,
    pub proxy_dir: Option<String>,
    pub target_url: Option<String>,
    pub cache: Option<i32>,
    pub status: Option<String>,
}

/// 列出站点的反向代理
pub async fn list_proxies(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
) -> Json<serde_json::Value> {
    match reverse_proxy_service::list_by_site(&state, site_id).await {
        Ok(proxies) => Json(json!(ApiResponse::success(proxies))),
        Err(e) => Json(json!(ApiResponse::<()>::error(&e.to_string()))),
    }
}

/// 创建反向代理
#[audit_log(module = "proxy", action = "创建反向代理", capture = req)]
pub async fn create_proxy(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
    Json(req): Json<CreateProxyRequest>,
) -> Json<serde_json::Value> {
    let proxy_dir = req.proxy_dir.as_deref().unwrap_or("/");
    match reverse_proxy_service::create(&state, site_id, &req.name, proxy_dir, &req.target_url, req.cache.unwrap_or(0)).await {
        Ok(proxy) => {
            regenerate_and_reload(&state, site_id).await;
            Json(json!(ApiResponse::success(proxy)))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(&e.to_string()))),
    }
}

/// 更新反向代理
#[audit_log(module = "proxy", action = "更新反向代理", capture = req)]
pub async fn update_proxy(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateProxyRequest>,
) -> Json<serde_json::Value> {
    // 先查 site_id
    let site_id = match get_proxy_site_id(&state, id).await {
        Some(sid) => sid,
        None => return Json(json!(ApiResponse::<()>::error("代理不存在"))),
    };

    match reverse_proxy_service::update(
        &state,
        id,
        req.name.as_deref(),
        req.proxy_dir.as_deref(),
        req.target_url.as_deref(),
        req.cache,
        req.status.as_deref(),
    )
    .await
    {
        Ok(Some(proxy)) => {
            regenerate_and_reload(&state, site_id).await;
            Json(json!(ApiResponse::success(proxy)))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("代理不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(&e.to_string()))),
    }
}

/// 删除反向代理
#[audit_log(module = "proxy", action = "删除反向代理")]
pub async fn delete_proxy(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    let site_id = match get_proxy_site_id(&state, id).await {
        Some(sid) => sid,
        None => return Json(json!(ApiResponse::<()>::error("代理不存在"))),
    };

    match reverse_proxy_service::delete(&state, id).await {
        Ok(true) => {
            regenerate_and_reload(&state, site_id).await;
            Json(json!(ApiResponse::success("删除成功")))
        }
        Ok(false) => Json(json!(ApiResponse::<()>::error("代理不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(&e.to_string()))),
    }
}

async fn get_proxy_site_id(state: &AppState, proxy_id: i64) -> Option<i64> {
    sqlx::query_scalar::<_, i64>("SELECT site_id FROM sys_reverse_proxies WHERE id = ?")
        .bind(proxy_id)
        .fetch_optional(state.db.pool())
        .await
        .ok()
        .flatten()
}

async fn regenerate_and_reload(state: &AppState, site_id: i64) {
    let config = state.get_config();
    if let Ok(Some(site)) = crate::service::site_service::get_site(state, site_id).await {
        if site.status != "disabled" {
            let proxies = reverse_proxy_service::list_by_site(state, site_id).await.unwrap_or_default();
            let content = crate::nginx::generate_site_config_with_proxies(&site, &proxies);
            let _ = crate::nginx::write_site_config(&config.nginx.sites_enabled, &site.name, &content).await;
        }
        let _ = crate::nginx::reload_nginx(&config.nginx.bin).await;
    }
}
