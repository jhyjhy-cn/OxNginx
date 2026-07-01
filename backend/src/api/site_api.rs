use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::dto::{ApiResponse, CreateSiteRequest, DeleteSiteRequest, UpdateSiteRequest};
use crate::service::site_service;
use crate::AppState;

/// 批量操作请求
#[derive(Debug, Deserialize)]
pub struct BatchRequest {
    pub ids: Vec<i64>,
}

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
    Json(mut req): Json<CreateSiteRequest>,
) -> Json<serde_json::Value> {
    let config = state.get_config();

    // 如果未指定根目录，以站点名称自动创建
    if req.root_path.is_none() || req.root_path.as_deref() == Some("") {
        let auto_root = if cfg!(target_os = "linux") {
            format!("/var/www/html/{}", req.name)
        } else {
            let nginx_dir = std::path::Path::new(&config.nginx.bin)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            nginx_dir.join("html").join(&req.name).to_string_lossy().to_string()
        };
        req.root_path = Some(auto_root);
    }

    // 创建站点根目录并生成默认 index.html
    if let Some(ref root_path) = req.root_path {
        if let Err(e) = crate::nginx::create_default_index(root_path).await {
            return Json(json!(ApiResponse::<()>::error(format!("创建站点目录失败: {}", e))));
        }
    }

    // 确保 nginx.conf 包含 sites-enabled 的 include 指令
    let _ = crate::nginx::ensure_sites_enabled_include(&config.nginx.config, &config.nginx.sites_enabled).await;

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
    let sites_enabled = &config.nginx.sites_enabled;
    if let Err(e) = crate::nginx::write_site_config(sites_enabled, &req.name, &config_content).await {
        return Json(json!(ApiResponse::<()>::error(format!("写入配置文件失败: {}", e))));
    }

    // 测试配置
    let test_result = crate::nginx::test_config(&config.nginx.bin).await;
    if !test_result.success {
        // 回滚：删除配置文件
        let _ = crate::nginx::remove_site_config(sites_enabled, &req.name).await;
        return Json(json!(ApiResponse::<()>::error(format!("配置测试失败: {}", test_result.message))));
    }

    // 保存到数据库
    match site_service::create_site(&state, req).await {
        Ok(site) => {
            // 重载 nginx 配置
            let _ = crate::nginx::reload_nginx(&config.nginx.bin).await;
            Json(json!(ApiResponse::success(site)))
        }
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
            let config = state.get_config();
            let sites_enabled = &config.nginx.sites_enabled;

            if site.status == "disabled" {
                // 禁用：删除配置文件
                let _ = crate::nginx::remove_site_config(sites_enabled, &site.name).await;
            } else {
                // 启用：写入配置文件
                let config_content = crate::nginx::generate_site_config(&site);
                let _ = crate::nginx::write_site_config(sites_enabled, &site.name, &config_content).await;
            }

            // 重载 nginx 配置
            let _ = crate::nginx::reload_nginx(&config.nginx.bin).await;
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
    Json(req): Json<DeleteSiteRequest>,
) -> Json<serde_json::Value> {
    // 先获取站点信息
    let site = match site_service::get_site(&state, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    };

    let config = state.get_config();
    let sites_enabled = &config.nginx.sites_enabled;

    // 删除配置文件（总是删除）
    let _ = crate::nginx::remove_site_config(sites_enabled, &site.name).await;

    // 删除站点文件目录
    if req.delete_files {
        if let Some(ref root_path) = site.root_path {
            if !root_path.is_empty() {
                #[cfg(target_os = "linux")]
                {
                    let cmd = format!("sudo rm -rf '{}'", root_path);
                    let _ = tokio::process::Command::new("sh")
                        .args(["-c", &cmd])
                        .output()
                        .await;
                }
                #[cfg(target_os = "windows")]
                {
                    let _ = tokio::fs::remove_dir_all(root_path).await;
                }
            }
        }
    }

    // 删除数据库记录
    if req.delete_record {
        match site_service::delete_site(&state, id).await {
            Ok(true) => {
                // 重载 nginx 配置
                let _ = crate::nginx::reload_nginx(&config.nginx.bin).await;
                Json(json!(ApiResponse::success("站点已删除")))
            }
            Ok(false) => Json(json!(ApiResponse::<()>::error("删除站点失败"))),
            Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除站点失败: {}", e)))),
        }
    } else {
        // 只删除文件，保留记录
        let _ = crate::nginx::reload_nginx(&config.nginx.bin).await;
        Json(json!(ApiResponse::success("站点文件已删除")))
    }
}

/// 批量启用站点
pub async fn batch_enable(
    State(state): State<AppState>,
    Json(req): Json<BatchRequest>,
) -> Json<serde_json::Value> {
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
            status: Some("enabled".to_string()),
        };

        match site_service::update_site(&state, *id, update_req).await {
            Ok(Some(site)) => {
                let config_content = crate::nginx::generate_site_config(&site);
                let config = state.get_config();
                let sites_enabled = &config.nginx.sites_enabled;
                let _ = crate::nginx::write_site_config(sites_enabled, &site.name, &config_content).await;
                success_count += 1;
            }
            _ => error_count += 1,
        }
    }

    // 重载 nginx 配置
    let config = state.get_config();
    let _ = crate::nginx::reload_nginx(&config.nginx.bin).await;

    Json(json!(ApiResponse::success(serde_json::json!({
        "success": success_count,
        "error": error_count,
    }))))
}

/// 批量禁用站点
pub async fn batch_disable(
    State(state): State<AppState>,
    Json(req): Json<BatchRequest>,
) -> Json<serde_json::Value> {
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
            status: Some("disabled".to_string()),
        };

        match site_service::update_site(&state, *id, update_req).await {
            Ok(Some(site)) => {
                let config = state.get_config();
                let sites_enabled = &config.nginx.sites_enabled;
                let _ = crate::nginx::remove_site_config(sites_enabled, &site.name).await;
                success_count += 1;
            }
            _ => error_count += 1,
        }
    }

    // 重载 nginx 配置
    let config = state.get_config();
    let _ = crate::nginx::reload_nginx(&config.nginx.bin).await;

    Json(json!(ApiResponse::success(serde_json::json!({
        "success": success_count,
        "error": error_count,
    }))))
}

/// 批量删除站点
pub async fn batch_delete(
    State(state): State<AppState>,
    Json(req): Json<BatchRequest>,
) -> Json<serde_json::Value> {
    let mut success_count = 0;
    let mut error_count = 0;

    for id in &req.ids {
        // 先获取站点信息
        let site = match site_service::get_site(&state, *id).await {
            Ok(Some(s)) => s,
            _ => {
                error_count += 1;
                continue;
            }
        };

        // 删除配置文件
        let config = state.get_config();
        let sites_enabled = &config.nginx.sites_enabled;
        let _ = crate::nginx::remove_site_config(sites_enabled, &site.name).await;

        // 删除数据库记录
        match site_service::delete_site(&state, *id).await {
            Ok(true) => success_count += 1,
            _ => error_count += 1,
        }
    }

    Json(json!(ApiResponse::success(serde_json::json!({
        "success": success_count,
        "error": error_count,
    }))))
}
