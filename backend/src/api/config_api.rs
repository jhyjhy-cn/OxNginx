use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::dto::ApiResponse;
use crate::AppState;

/// 配置文件内容
#[derive(Debug, Serialize)]
pub struct ConfigContent {
    pub path: String,
    pub content: String,
}

/// 保存配置请求
#[derive(Debug, Deserialize)]
pub struct SaveConfigRequest {
    pub content: String,
}

/// 列出配置文件
pub async fn list_config_files(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sites_available = format!("{}/../sites-available", state.config.nginx.sites_enabled);
    let sites_enabled = &state.config.nginx.sites_enabled;

    let mut files = Vec::new();

    // 列出 sites-available 目录
    if let Ok(entries) = std::fs::read_dir(&sites_available) {
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

                // 检查是否已启用
                let enabled_path = format!("{}/{}", sites_enabled, name);
                let enabled = std::path::Path::new(&enabled_path).exists();

                files.push(serde_json::json!({
                    "name": name,
                    "path": path.to_string_lossy(),
                    "size": size,
                    "modified": modified,
                    "enabled": enabled,
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
    let config_path = &state.config.nginx.config;
    match tokio::fs::read_to_string(config_path).await {
        Ok(content) => Json(json!(ApiResponse::success(ConfigContent {
            path: config_path.clone(),
            content,
        }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取配置文件失败: {}", e)))),
    }
}

/// 保存主配置文件
pub async fn save_main_config(
    State(state): State<AppState>,
    Json(req): Json<SaveConfigRequest>,
) -> Json<serde_json::Value> {
    let config_path = &state.config.nginx.config;

    // 备份原配置
    let backup_path = format!("{}.bak.{}", config_path, chrono::Local::now().format("%Y%m%d%H%M%S"));
    if let Ok(content) = tokio::fs::read_to_string(config_path).await {
        let _ = tokio::fs::write(&backup_path, content).await;
    }

    // 保存新配置
    match tokio::fs::write(config_path, &req.content).await {
        Ok(_) => {
            // 测试配置
            let test_result = crate::nginx::test_config(&state.config.nginx.bin).await;
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
    let sites_available = format!("{}/../sites-available", state.config.nginx.sites_enabled);
    let config_path = format!("{}/{}", sites_available, name);

    match tokio::fs::read_to_string(&config_path).await {
        Ok(content) => Json(json!(ApiResponse::success(ConfigContent {
            path: config_path,
            content,
        }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取配置文件失败: {}", e)))),
    }
}

/// 保存站点配置文件
pub async fn save_site_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<SaveConfigRequest>,
) -> Json<serde_json::Value> {
    let sites_available = format!("{}/../sites-available", state.config.nginx.sites_enabled);
    let config_path = format!("{}/{}", sites_available, name);

    // 备份原配置
    let backup_path = format!("{}.bak.{}", config_path, chrono::Local::now().format("%Y%m%d%H%M%S"));
    if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
        let _ = tokio::fs::write(&backup_path, content).await;
    }

    // 保存新配置
    match tokio::fs::write(&config_path, &req.content).await {
        Ok(_) => {
            // 测试配置
            let test_result = crate::nginx::test_config(&state.config.nginx.bin).await;
            if test_result.success {
                Json(json!(ApiResponse::success("配置保存成功")))
            } else {
                // 配置测试失败，恢复备份
                if let Ok(backup_content) = tokio::fs::read_to_string(&backup_path).await {
                    let _ = tokio::fs::write(&config_path, backup_content).await;
                }
                Json(json!(ApiResponse::<()>::error(format!("配置测试失败，已回滚: {}", test_result.message))))
            }
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("保存配置文件失败: {}", e)))),
    }
}

/// 启用/禁用站点配置
pub async fn toggle_site_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<serde_json::Value> {
    let sites_available = format!("{}/../sites-available", state.config.nginx.sites_enabled);
    let sites_enabled = &state.config.nginx.sites_enabled;

    let source = format!("{}/{}", sites_available, name);
    let target = format!("{}/{}", sites_enabled, name);

    if !std::path::Path::new(&source).exists() {
        return Json(json!(ApiResponse::<()>::error("配置文件不存在")));
    }

    let target_path = std::path::Path::new(&target);
    if target_path.exists() {
        // 已启用，禁用它
        match tokio::fs::remove_file(&target).await {
            Ok(_) => Json(json!(ApiResponse::success("配置已禁用"))),
            Err(e) => Json(json!(ApiResponse::<()>::error(format!("禁用失败: {}", e)))),
        }
    } else {
        // 未启用，启用它（复制文件而非符号链接，兼容 Windows）
        match tokio::fs::copy(&source, &target).await {
            Ok(_) => {
                // 测试配置
                let test_result = crate::nginx::test_config(&state.config.nginx.bin).await;
                if test_result.success {
                    Json(json!(ApiResponse::success("配置已启用")))
                } else {
                    // 配置测试失败，移除复制的文件
                    let _ = tokio::fs::remove_file(&target).await;
                    Json(json!(ApiResponse::<()>::error(format!("配置测试失败: {}", test_result.message))))
                }
            }
            Err(e) => Json(json!(ApiResponse::<()>::error(format!("启用失败: {}", e)))),
        }
    }
}

/// 删除站点配置文件
pub async fn delete_site_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<serde_json::Value> {
    let sites_available = format!("{}/../sites-available", state.config.nginx.sites_enabled);
    let sites_enabled = &state.config.nginx.sites_enabled;

    let source = format!("{}/{}", sites_available, name);
    let target = format!("{}/{}", sites_enabled, name);

    // 删除符号链接
    if std::path::Path::new(&target).exists() {
        let _ = tokio::fs::remove_file(&target).await;
    }

    // 删除源文件
    match tokio::fs::remove_file(&source).await {
        Ok(_) => Json(json!(ApiResponse::success("配置文件已删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除失败: {}", e)))),
    }
}
