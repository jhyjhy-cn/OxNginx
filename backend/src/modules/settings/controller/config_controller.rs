use axum::Extension;
use crate::modules::common::audit::context::SharedAuditContext;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::ApiResponse;
use crate::AppState;

/// 配置文件内容
#[derive(Debug, Serialize)]
pub struct ConfigContent {
    pub path: String,
    pub content: String,
}

/// 保存配置请求
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveConfigRequest {
    pub content: String,
}

/// 列出配置文件
pub async fn list_config_files(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let sites_enabled = &config.nginx.sites_enabled;

    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(sites_enabled) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "conf") {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let metadata = std::fs::metadata(&path).ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| {
                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                        Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
                    });

                files.push(serde_json::json!({
                    "name": name,
                    "path": path.to_string_lossy(),
                    "size": size,
                    "modified": modified,
                    "enabled": true,
                }));
            }
        }
    }

    Json(json!(ApiResponse::success(files)))
}

/// 读取主配置文件
pub async fn get_main_config(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let config_path = &config.nginx.config;
    match tokio::fs::read_to_string(config_path).await {
        Ok(content) => Json(json!(ApiResponse::success(ConfigContent {
            path: config_path.clone(),
            content,
        }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取配置文件失败: {}", e)))),
    }
}

/// 保存主配置文件
#[audit_log(module = "config", action = "保存主配置", capture = req)]
pub async fn save_main_config(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Json(req): Json<SaveConfigRequest>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let config_path = &config.nginx.config;

    // 备份原配置
    let backup_path = format!("{}.bak.{}", config_path, chrono::Local::now().format("%Y%m%d%H%M%S"));
    if let Ok(content) = tokio::fs::read_to_string(config_path).await {
        let _ = tokio::fs::write(&backup_path, content).await;
    }

    // 保存新配置
    match tokio::fs::write(config_path, &req.content).await {
        Ok(_) => {
            // 测试配置
            let test_result = crate::modules::common::nginx::test_config(&config.nginx.bin).await;
            if test_result.success {
                Json(json!(ApiResponse::success("配置保存成功")))
            } else {
                // 配置测试失败，恢复备份
                if let Ok(backup_content) = tokio::fs::read_to_string(&backup_path).await {
                    let _ = tokio::fs::write(config_path, backup_content).await;
                }
                Json(json!(ApiResponse::<()>::error(format!("配置测试失败，已回滚: {}", test_result.message))))
            }
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("保存配置文件失败: {}", e)))),
    }
}

/// 读取站点配置文件
pub async fn get_site_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let conf_name = if name.ends_with(".conf") { name } else { format!("{}.conf", name) };
    let config_path = format!("{}/{}", config.nginx.sites_enabled, conf_name);
    tracing::info!("读取站点配置: {}", config_path);

    match tokio::fs::read_to_string(&config_path).await {
        Ok(content) => Json(json!(ApiResponse::success(ConfigContent {
            path: config_path,
            content,
        }))),
        Err(e) => {
            tracing::error!("读取配置文件失败: {} - {}", config_path, e);
            Json(json!(ApiResponse::<()>::error(format!("读取配置文件失败: {}", e))))
        }
    }
}

/// 保存站点配置文件
#[audit_log(module = "config", action = "保存站点配置", capture = req)]
pub async fn save_site_config(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<SaveConfigRequest>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let conf_name = if name.ends_with(".conf") { name } else { format!("{}.conf", name) };
    let config_path = format!("{}/{}", config.nginx.sites_enabled, conf_name);
    tracing::info!("保存站点配置: {}", config_path);

    // 备份原配置
    let backup_path = format!("{}.bak.{}", config_path, chrono::Local::now().format("%Y%m%d%H%M%S"));
    if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
        let _ = tokio::fs::write(&backup_path, content).await;
    }

    // 保存新配置
    match tokio::fs::write(&config_path, &req.content).await {
        Ok(_) => {
            // 测试配置
            let test_result = crate::modules::common::nginx::test_config(&config.nginx.bin).await;
            if test_result.success {
                Json(json!(ApiResponse::success("配置保存成功")))
            } else {
                tracing::error!("配置测试失败，已回滚: {}", test_result.message);
                // 配置测试失败，恢复备份
                if let Ok(backup_content) = tokio::fs::read_to_string(&backup_path).await {
                    let _ = tokio::fs::write(&config_path, backup_content).await;
                }
                Json(json!(ApiResponse::<()>::error(format!("配置测试失败，已回滚: {}", test_result.message))))
            }
        }
        Err(e) => {
            tracing::error!("保存配置文件失败: {} - {}", config_path, e);
            Json(json!(ApiResponse::<()>::error(format!("保存配置文件失败: {}", e))))
        }
    }
}

/// 启用/禁用站点配置
#[audit_log(module = "config", action = "启禁站点配置")]
pub async fn toggle_site_config(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let conf_name = if name.ends_with(".conf") { name } else { format!("{}.conf", name) };
    let config_path = format!("{}/{}", config.nginx.sites_enabled, conf_name);

    if !std::path::Path::new(&config_path).exists() {
        tracing::error!("配置文件不存在: {}", config_path);
        return Json(json!(ApiResponse::<()>::error("配置文件不存在")));
    }

    match tokio::fs::remove_file(&config_path).await {
        Ok(_) => Json(json!(ApiResponse::success("配置已禁用"))),
        Err(e) => {
            tracing::error!("禁用配置失败: {} - {}", config_path, e);
            Json(json!(ApiResponse::<()>::error(format!("禁用失败: {}", e))))
        }
    }
}

/// 删除站点配置文件
#[audit_log(module = "config", action = "删除站点配置")]
pub async fn delete_site_config(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let conf_name = if name.ends_with(".conf") { name } else { format!("{}.conf", name) };
    let config_path = format!("{}/{}", config.nginx.sites_enabled, conf_name);

    match tokio::fs::remove_file(&config_path).await {
        Ok(_) => Json(json!(ApiResponse::success("配置文件已删除"))),
        Err(e) => {
            tracing::error!("删除配置文件失败: {} - {}", config_path, e);
            Json(json!(ApiResponse::<()>::error(format!("删除失败: {}", e))))
        }
    }
}
