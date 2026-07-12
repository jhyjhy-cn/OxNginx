use axum::Extension;
use crate::modules::common::audit::context::SharedAuditContext;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::{ApiResponse, CreateSiteRequest, DeleteSiteRequest, UpdateSiteRequest};
use crate::modules::common::nginx::get_nginx_config;
use crate::modules::site::service::site_service;
use crate::modules::sys::service::param_service::NginxConfigFromDb;
use crate::AppState;

/// 批量操作请求
#[derive(Debug, Deserialize, Serialize)]
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

/// 获取站点列表（含证书信息）
pub async fn list_sites_with_certs(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match site_service::get_all_sites(&state).await {
        Ok(sites) => {
            // 一次性获取所有站点的备份数量
            let site_names: Vec<String> = sites.iter().map(|s| s.name.clone()).collect();
            let backup_counts = crate::modules::site::service::site_backup_service::get_backup_counts(&site_names);

            let mut result = Vec::new();
            for site in sites {
                let (cert_expire_time, cert_expire_days) = if site.ssl == 1 {
                    let cert_info = site.get_cert_expire_info().await;
                    (
                        cert_info.as_ref().and_then(|c| c.expire_time.clone()),
                        cert_info.as_ref().and_then(|c| c.days_remaining),
                    )
                } else {
                    (None, None)
                };

                let json = serde_json::json!({
                    "id": site.id,
                    "name": site.name,
                    "server_name": site.server_name,
                    "listen": site.listen,
                    "ssl": site.ssl,
                    "certificate_path": site.certificate_path,
                    "key_path": site.key_path,
                    "proxy_pass": site.proxy_pass,
                    "root_path": site.root_path,
                    "config": site.config,
                    "remark": site.remark,
                    "expire_time": site.expire_time,
                    "rewrite_rules": site.rewrite_rules,
                    "redirect_rules": site.redirect_rules,
                    "hotlink_config": site.hotlink_config,
                    "log_access_path": site.log_access_path,
                    "log_error_path": site.log_error_path,
                    "status": site.status,
                    "created_at": site.created_at,
                    "updated_at": site.updated_at,
                    "cert_expire_time": cert_expire_time,
                    "cert_expire_days": cert_expire_days,
                    "backup_count": backup_counts.get(&site.name).copied().unwrap_or(0),
                });
                result.push(json);
            }
            Json(json!(ApiResponse::success(result)))
        }
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
#[audit_log(module = "site", action = "创建站点", capture = req)]
pub async fn create_site(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Json(mut req): Json<CreateSiteRequest>,
) -> Json<serde_json::Value> {
    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };

    // 归一化 server_name：换行符转空格，去首尾空白
    req.server_name = req.server_name
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // 如果未指定根目录，以站点名称自动创建
    if req.root_path.is_none() || req.root_path.as_deref() == Some("") {
        let default_root = nginx_config.default_root.as_deref().unwrap_or("");
        if default_root.is_empty() {
            return Json(json!(ApiResponse::<()>::error("默认站点根目录未设置，请检查系统参数")));
        }
        let auto_root = format!("{}/{}", default_root, req.name);
        req.root_path = Some(auto_root);
    }

    // 创建站点根目录并生成默认 index.html
    if let Some(ref root_path) = req.root_path {
        if let Err(e) = crate::modules::common::nginx::create_default_index(root_path).await {
            return Json(json!(ApiResponse::<()>::error(format!("创建站点目录失败: {}", e))));
        }
    }

    // 确保 nginx.conf 包含 sites-enabled 的 include 指令
    if let (Some(config_path), Some(sites_enabled)) = (&nginx_config.config, &nginx_config.sites_enabled) {
        let _ = crate::modules::common::nginx::ensure_sites_enabled_include(config_path, sites_enabled).await;
    }

    // 生成配置
    let site_model = crate::modules::site::entity::site::Site {
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
        remark: req.remark.clone(),
        expire_time: req.expire_time.clone(),
        rewrite_rules: req.rewrite_rules.clone(),
        redirect_rules: req.redirect_rules.clone(),
        hotlink_config: req.hotlink_config.clone(),
        log_access_path: req.log_access_path.clone(),
        log_error_path: req.log_error_path.clone(),
        sort: 0,
        version: 0,
        dept_id: None,
        is_deleted: 0,
        created_by: None,
        updated_by: None,
        status: 1,
        created_at: None,
        updated_at: None,
    };
    let config_content = crate::modules::common::nginx::generate_site_config(&site_model);

    // 备份并写入配置
    let sites_enabled = match &nginx_config.sites_enabled {
        Some(p) if !p.is_empty() => p,
        _ => return Json(json!(ApiResponse::<()>::error("站点配置目录未设置，请检查系统参数"))),
    };
    if let Err(e) = crate::modules::common::nginx::write_site_config(sites_enabled, &req.name, &config_content).await {
        return Json(json!(ApiResponse::<()>::error(format!("写入配置文件失败: {}", e))));
    }

    let nginx_bin = match &nginx_config.bin {
        Some(p) if !p.is_empty() => p,
        _ => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
    };

    // 测试配置
    let test_result = crate::modules::common::nginx::test_config(nginx_bin).await;
    if !test_result.success {
        // 回滚：删除配置文件
        let _ = crate::modules::common::nginx::remove_site_config(sites_enabled, &req.name).await;
        return Json(json!(ApiResponse::<()>::error(format!("配置测试失败: {}", test_result.message))));
    }

    // 保存到数据库
    match site_service::create_site(&state, req).await {
        Ok(site) => {
            let _ = crate::modules::common::nginx::reload_nginx(nginx_bin).await;
            Json(json!(ApiResponse::success(site)))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建站点失败: {}", e)))),
    }
}

/// 更新站点
#[audit_log(module = "site", action = "更新站点", capture = req)]
pub async fn update_site(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSiteRequest>,
) -> Json<serde_json::Value> {
    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };

    match site_service::update_site(&state, id, req).await {
        Ok(Some(site)) => {
            let sites_enabled = nginx_config.sites_enabled.as_deref().unwrap_or("");
            let nginx_bin = nginx_config.bin.as_deref().unwrap_or("");

            if site.status == 0 {
                // 禁用：删除配置文件
                let _ = crate::modules::common::nginx::remove_site_config(sites_enabled, &site.name).await;
            } else {
                // 启用：写入配置文件
                let config_content = crate::modules::common::nginx::generate_site_config(&site);
                let _ = crate::modules::common::nginx::write_site_config(sites_enabled, &site.name, &config_content).await;
            }

            // 重载 nginx 配置
            if !nginx_bin.is_empty() {
                let _ = crate::modules::common::nginx::reload_nginx(nginx_bin).await;
            }
            Json(json!(ApiResponse::success(site)))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("更新站点失败: {}", e)))),
    }
}

/// 删除站点
#[audit_log(module = "site", action = "删除站点", capture = req)]
pub async fn delete_site(
    ctx: Extension<SharedAuditContext>,

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

    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    let sites_enabled = nginx_config.sites_enabled.as_deref().unwrap_or("");
    let nginx_bin = nginx_config.bin.as_deref().unwrap_or("");

    // 删除配置文件（总是删除）
    let _ = crate::modules::common::nginx::remove_site_config(sites_enabled, &site.name).await;

    // 删除站点文件目录
    if req.delete_files {
        if let Some(ref root_path) = site.root_path {
            if !root_path.is_empty() {
                // root 用户直接删除，无需 sudo
                let _ = tokio::fs::remove_dir_all(root_path).await;
            }
        }
    }

    // 删除数据库记录
    if req.delete_record {
        match site_service::delete_site(&state, id).await {
            Ok(true) => {
                // 重载 nginx 配置
                if !nginx_bin.is_empty() {
                    let _ = crate::modules::common::nginx::reload_nginx(nginx_bin).await;
                }
                Json(json!(ApiResponse::success("站点已删除")))
            }
            Ok(false) => Json(json!(ApiResponse::<()>::error("删除站点失败"))),
            Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除站点失败: {}", e)))),
        }
    } else {
        // 只删除文件，保留记录
        if !nginx_bin.is_empty() {
            let _ = crate::modules::common::nginx::reload_nginx(nginx_bin).await;
        }
        Json(json!(ApiResponse::success("站点文件已删除")))
    }
}

/// 批量启用站点
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

/// 部署SSL证书（一键申请Let's Encrypt并绑定到站点）
#[audit_log(module = "site", action = "部署SSL证书")]
pub async fn deploy_ssl(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    let nginx_bin = match nginx_config.bin.as_deref() {
        Some(b) if !b.is_empty() => b,
        _ => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
    };
    let nginx_config_path = match nginx_config.config.as_deref() {
        Some(c) if !c.is_empty() => c,
        _ => return Json(json!(ApiResponse::<()>::error("Nginx配置文件路径未设置"))),
    };

    let site = match site_service::get_site(&state, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    };

    // 停止nginx（释放80端口给standalone模式）
    let _ = crate::modules::common::nginx::stop_nginx(nginx_bin).await;

    // 申请证书
    let cert = match crate::modules::site::service::cert_service::apply_cert(&state, &site.server_name).await {
        Ok(c) => c,
        Err(e) => {
            let _ = crate::modules::common::nginx::start_nginx(nginx_bin, nginx_config_path).await;
            return Json(json!(ApiResponse::<()>::error(format!("证书申请失败: {}", e))));
        }
    };

    // 重启nginx
    let _ = crate::modules::common::nginx::start_nginx(nginx_bin, nginx_config_path).await;

    let cert_domain = cert.domain.clone();
    let cert_src = cert.cert_path.clone().unwrap_or_default();
    let key_src = cert.key_path.clone().unwrap_or_default();
    let expire_time = cert.expire_time.clone();

    // 将证书复制到 nginx 可读的位置
    let ssl_dir = match &nginx_config.ssl_dir {
        Some(d) if !d.is_empty() => format!("{}/{}", d, cert_domain),
        _ => return Json(json!(ApiResponse::<()>::error("SSL证书目录未设置"))),
    };
    let final_cert = format!("{}/fullchain.cer", ssl_dir);
    let final_key = format!("{}/private.key", ssl_dir);

    // root 用户直接操作，无需 sudo
    let _ = tokio::fs::create_dir_all(&ssl_dir).await;
    let copied = tokio::fs::copy(&cert_src, &final_cert).await.is_ok()
        && tokio::fs::copy(&key_src, &final_key).await.is_ok();

    // 设置权限
    if copied {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&final_cert, std::fs::Permissions::from_mode(0o644));
            let _ = std::fs::set_permissions(&final_key, std::fs::Permissions::from_mode(0o640));
        }
    }

    let cert_path = if copied { final_cert } else { cert_src };
    let key_path = if copied { final_key } else { key_src };

    // 更新站点SSL配置
    let update_req = crate::modules::common::dto::UpdateSiteRequest {
        name: None,
        server_name: None,
        listen: None,
        ssl: Some(true),
        certificate_path: Some(Some(cert_path.to_string())),
        key_path: Some(Some(key_path.to_string())),
        proxy_pass: None,
        root_path: None,
        remark: None,
        expire_time: None,
        rewrite_rules: None,
        redirect_rules: None,
        hotlink_config: None,
        log_access_path: None,
        log_error_path: None,
        status: None,
    };

    let updated_site = match site_service::update_site(&state, id, update_req).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("更新站点失败"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("更新站点失败: {}", e)))),
    };

    let sites_enabled = nginx_config.sites_enabled.as_deref().unwrap_or("");

    // 生成并写入nginx配置
    let config_content = crate::modules::common::nginx::generate_site_config(&updated_site);
    if let Err(e) = crate::modules::common::nginx::write_site_config(sites_enabled, &site.name, &config_content).await {
        return Json(json!(ApiResponse::<()>::error(format!("写入配置失败: {}", e))));
    }

    // 测试配置
    let test_result = crate::modules::common::nginx::test_config(nginx_bin).await;
    if !test_result.success {
        let _ = crate::modules::common::nginx::remove_site_config(sites_enabled, &site.name).await;
        return Json(json!(ApiResponse::<()>::error(format!("配置测试失败: {}", test_result.message))));
    }

    // 重载nginx
    let _ = crate::modules::common::nginx::reload_nginx(nginx_bin).await;

    Json(json!(ApiResponse::success(serde_json::json!({
        "domain": cert_domain,
        "cert_path": cert_path,
        "key_path": key_path,
        "expire_time": expire_time,
    }))))
}



/// 批量删除站点
#[audit_log(module = "site", action = "批量删除站点", capture = req)]
pub async fn batch_delete(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Json(req): Json<BatchRequest>,
) -> Json<serde_json::Value> {
    let nginx_config = match get_nginx_config(&state).await {
        Ok(cfg) => cfg,
        Err(_) => NginxConfigFromDb::default(),
    };
    let sites_enabled = nginx_config.sites_enabled.as_deref().unwrap_or("");

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
        let _ = crate::modules::common::nginx::remove_site_config(sites_enabled, &site.name).await;

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
