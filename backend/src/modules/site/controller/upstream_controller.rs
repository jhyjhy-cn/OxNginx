use axum::Extension;
use crate::modules::common::audit::context::SharedAuditContext;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::{ApiResponse, CreateUpstreamRequest, UpdateUpstreamRequest};
use crate::modules::common::nginx::get_nginx_config;
use crate::modules::site::service::upstream_service;
use crate::AppState;

/// 获取上游服务器列表
pub async fn list_upstreams(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match upstream_service::get_all_upstreams(&state).await {
        Ok(upstreams) => Json(json!(ApiResponse::success(upstreams))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取上游服务器列表失败: {}", e)))),
    }
}

/// 获取单个上游服务器
pub async fn get_upstream(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match upstream_service::get_upstream_with_servers(&state, id).await {
        Ok(Some((upstream, servers))) => {
            Json(json!(ApiResponse::success(serde_json::json!({
                "upstream": upstream,
                "servers": servers,
            }))))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("上游服务器不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取上游服务器失败: {}", e)))),
    }
}

/// 创建上游服务器
#[audit_log(module = "upstream", action = "创建上游服务器", capture = req)]
pub async fn create_upstream(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Json(req): Json<CreateUpstreamRequest>,
) -> Json<serde_json::Value> {
    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    let nginx_bin = match nginx_config.bin.as_deref() {
        Some(b) if !b.is_empty() => b,
        _ => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
    };

    // 生成配置并测试
    let upstream_config = crate::modules::common::nginx::generate_upstream_config_from_request(&req);
    let test_result = crate::modules::common::nginx::test_config(nginx_bin).await;
    if !test_result.success {
        return Json(json!(ApiResponse::<()>::error(format!("配置测试失败: {}", test_result.message))));
    }

    match upstream_service::create_upstream(&state, req).await {
        Ok((upstream, servers)) => {
            // 写入配置文件
            if let Some(sites_enabled) = &nginx_config.sites_enabled {
                let config_dir = format!("{}/../conf.d", sites_enabled);
                let config_path = format!("{}/upstream-{}.conf", config_dir, upstream.name);
                let _ = tokio::fs::write(&config_path, &upstream_config).await;
            }

            Json(json!(ApiResponse::success(serde_json::json!({
                "upstream": upstream,
                "servers": servers,
            }))))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建上游服务器失败: {}", e)))),
    }
}

/// 更新上游服务器
#[audit_log(module = "upstream", action = "更新上游服务器", capture = req)]
pub async fn update_upstream(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUpstreamRequest>,
) -> Json<serde_json::Value> {
    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };

    match upstream_service::update_upstream(&state, id, req).await {
        Ok(Some((upstream, servers))) => {
            // 重新生成配置
            let upstream_config = crate::modules::common::nginx::generate_upstream_config(&upstream, &servers);
            if let Some(sites_enabled) = &nginx_config.sites_enabled {
                let config_dir = format!("{}/../conf.d", sites_enabled);
                let config_path = format!("{}/upstream-{}.conf", config_dir, upstream.name);
                let _ = tokio::fs::write(&config_path, &upstream_config).await;
            }

            Json(json!(ApiResponse::success(serde_json::json!({
                "upstream": upstream,
                "servers": servers,
            }))))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("上游服务器不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("更新上游服务器失败: {}", e)))),
    }
}

/// 删除上游服务器
#[audit_log(module = "upstream", action = "删除上游服务器")]
pub async fn delete_upstream(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    let _ = ctx;
    // 先获取上游服务器信息
    let upstream = match upstream_service::get_upstream(&state, id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("上游服务器不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取上游服务器失败: {}", e)))),
    };

    // 删除配置文件
    if let Ok(nginx_config) = get_nginx_config(&state).await {
        if let Some(sites_enabled) = &nginx_config.sites_enabled {
            let config_dir = format!("{}/../conf.d", sites_enabled);
            let config_path = format!("{}/upstream-{}.conf", config_dir, upstream.name);
            let _ = tokio::fs::remove_file(&config_path).await;
        }
    }

    // 删除数据库记录
    match upstream_service::delete_upstream(&state, id).await {
        Ok(true) => Json(json!(ApiResponse::success("上游服务器已删除"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("删除上游服务器失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除上游服务器失败: {}", e)))),
    }
}
