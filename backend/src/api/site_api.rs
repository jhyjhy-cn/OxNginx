use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use crate::dto::{ApiResponse, CreateSiteRequest, UpdateSiteRequest};
use crate::service::site_service;
use crate::AppState;

/// 获取站点列表
pub async fn list_sites(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match site_service::get_all_sites(&state).await {
        Ok(sites) => Json(json!(ApiResponse::success(sites))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取站点列表失败: {}", e)))),
    }
}

/// 获取单个站点
pub async fn get_site(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match site_service::get_site(&state, id).await {
        Ok(Some(site)) => Json(json!(ApiResponse::success(site))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    }
}

/// 创建站点
pub async fn create_site(
    State(state): State<AppState>,
    Json(req): Json<CreateSiteRequest>,
) -> Json<serde_json::Value> {
    // 生成配置
    let site_model = crate::model::Site {
        id: 0,
        name: req.name.clone(),
        server_name: req.server_name.clone(),
        listen: req.listen.clone(),
        ssl: if req.ssl { 1 } else { 0 },
        certificate_path: req.certificate_path.clone(),
        key_path: req.key_path.clone(),
        proxy_pass: req.proxy_pass.clone(),
        root_path: req.root_path.clone(),
        config: None,
        status: "enabled".into(),
        created_at: None,
        updated_at: None,
    };
    let config_content = crate::nginx::generate_site_config(&site_model);

    // 备份并写入配置
    let sites_enabled = &state.config.nginx.sites_enabled;
    if let Err(e) = crate::nginx::write_site_config(sites_enabled, &req.name, &config_content).await {
        return Json(json!(ApiResponse::<()>::error(format!("写入配置文件失败: {}", e))));
    }

    // 测试配置
    let test_result = crate::nginx::test_config(&state.config.nginx.bin).await;
    if !test_result.success {
        // 回滚：删除配置文件
        let _ = crate::nginx::remove_site_config(sites_enabled, &req.name).await;
        return Json(json!(ApiResponse::<()>::error(format!("配置测试失败: {}", test_result.message))));
    }

    // 保存到数据库
    match site_service::create_site(&state, req).await {
        Ok(site) => Json(json!(ApiResponse::success(site))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建站点失败: {}", e)))),
    }
}

/// 更新站点
pub async fn update_site(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSiteRequest>,
) -> Json<serde_json::Value> {
    match site_service::update_site(&state, id, req).await {
        Ok(Some(site)) => {
            // 重新生成配置
            let config_content = crate::nginx::generate_site_config(&site);
            let sites_enabled = &state.config.nginx.sites_enabled;
            let _ = crate::nginx::write_site_config(sites_enabled, &site.name, &config_content).await;
            Json(json!(ApiResponse::success(site)))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("更新站点失败: {}", e)))),
    }
}

/// 删除站点
pub async fn delete_site(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    // 先获取站点信息
    let site = match site_service::get_site(&state, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    };

    // 删除配置文件
    let sites_enabled = &state.config.nginx.sites_enabled;
    let _ = crate::nginx::remove_site_config(sites_enabled, &site.name).await;

    // 删除数据库记录
    match site_service::delete_site(&state, id).await {
        Ok(true) => Json(json!(ApiResponse::success("站点已删除"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("删除站点失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除站点失败: {}", e)))),
    }
}
