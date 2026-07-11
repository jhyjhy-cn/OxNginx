use axum::{extract::State, Extension, Json};
use crate::modules::common::audit::context::SharedAuditContext;
use serde::{Deserialize, Serialize};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::ApiResponse;
use crate::modules::sys::service::param_service;
use crate::AppState;
use crate::modules::common::util::cmd;

/// 系统设置响应
#[derive(Debug, serde::Serialize)]
pub struct SettingsResponse {
    pub server: ServerSettings,
    pub nginx: NginxSettings,
    pub acme: AcmeSettings,
    pub system: SystemInfo,
}

#[derive(Debug, serde::Serialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, serde::Serialize)]
pub struct NginxSettings {
    pub bin: String,
    pub config: String,
    pub sites_enabled: String,
    pub ssl_dir: String,
    pub default_root: String,
    pub log_access: String,
    pub log_error: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AcmeSettings {
    pub bin: String,
    pub home: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub cpu_cores: usize,
    pub nginx_version: String,
    pub rust_version: String,
}

/// 更新设置请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateSettingsRequest {
    pub nginx_bin: Option<String>,
    pub nginx_config: Option<String>,
    pub nginx_sites_enabled: Option<String>,
    pub nginx_ssl_dir: Option<String>,
    pub nginx_default_root: Option<String>,
    pub nginx_log_access: Option<String>,
    pub nginx_log_error: Option<String>,
    pub acme_bin: Option<String>,
    pub acme_home: Option<String>,
}

/// 获取系统设置
pub async fn get_settings(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();

    // 从数据库获取 nginx 配置
    let nginx_db = match param_service::get_nginx_config(state.db.pool()).await {
        Ok(cfg) => cfg,
        Err(_) => param_service::NginxConfigFromDb::default(),
    };

    // 获取系统信息
    let hostname = get_hostname().unwrap_or_else(|_| "unknown".to_string());
    let nginx_bin = nginx_db.bin.as_deref().unwrap_or("");
    let nginx_version = get_nginx_version(nginx_bin).await;

    let settings = SettingsResponse {
        server: ServerSettings {
            host: config.server.host.clone(),
            port: config.server.port,
        },
        nginx: NginxSettings {
            bin: nginx_db.bin.unwrap_or_default(),
            config: nginx_db.config.unwrap_or_default(),
            sites_enabled: nginx_db.sites_enabled.unwrap_or_default(),
            ssl_dir: nginx_db.ssl_dir.unwrap_or_default(),
            default_root: nginx_db.default_root.unwrap_or_default(),
            log_access: nginx_db.log_access.unwrap_or_default(),
            log_error: nginx_db.log_error.unwrap_or_default(),
        },
        acme: AcmeSettings {
            bin: config.acme.bin.clone(),
            home: config.acme.home.clone(),
        },
        system: SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname,
            cpu_cores: num_cpus::get(),
            nginx_version,
            rust_version: "Stable".to_string(),
        },
    };

    Json(json!(ApiResponse::success(settings)))
}

/// 更新系统设置（nginx 配置写入 sys_params，acme 配置写入 config.toml）
#[audit_log(module = "system", action = "保存系统设置", capture = req)]
pub async fn update_settings(
    ctx: Extension<SharedAuditContext>,

    State(state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Json<serde_json::Value> {
    let pool = state.db.pool();

    // 更新 nginx 配置到 sys_params
    if let Some(nginx_bin) = &req.nginx_bin {
        if let Some(param) = param_service::get_param_by_key(pool, "nginx.bin").await.ok().flatten() {
            let _ = param_service::update_param(pool, param.id, Some(nginx_bin), None, None, None, None).await;
        }
    }
    if let Some(nginx_config) = &req.nginx_config {
        if let Some(param) = param_service::get_param_by_key(pool, "nginx.config").await.ok().flatten() {
            let _ = param_service::update_param(pool, param.id, Some(nginx_config), None, None, None, None).await;
        }
    }
    if let Some(sites_enabled) = &req.nginx_sites_enabled {
        if let Some(param) = param_service::get_param_by_key(pool, "nginx.sites_enabled").await.ok().flatten() {
            let _ = param_service::update_param(pool, param.id, Some(sites_enabled), None, None, None, None).await;
        }
    }
    if let Some(ssl_dir) = &req.nginx_ssl_dir {
        if let Some(param) = param_service::get_param_by_key(pool, "nginx.ssl_dir").await.ok().flatten() {
            let _ = param_service::update_param(pool, param.id, Some(ssl_dir), None, None, None, None).await;
        }
    }
    if let Some(default_root) = &req.nginx_default_root {
        if let Some(param) = param_service::get_param_by_key(pool, "nginx.default_root").await.ok().flatten() {
            let _ = param_service::update_param(pool, param.id, Some(default_root), None, None, None, None).await;
        }
    }
    if let Some(log_access) = &req.nginx_log_access {
        if let Some(param) = param_service::get_param_by_key(pool, "nginx.log_access").await.ok().flatten() {
            let _ = param_service::update_param(pool, param.id, Some(log_access), None, None, None, None).await;
        }
    }
    if let Some(log_error) = &req.nginx_log_error {
        if let Some(param) = param_service::get_param_by_key(pool, "nginx.log_error").await.ok().flatten() {
            let _ = param_service::update_param(pool, param.id, Some(log_error), None, None, None, None).await;
        }
    }

    // 更新 acme 配置到 config.toml
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".into());
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(mut config) = content.parse::<toml::Value>() {
            if let Some(acme) = config.get_mut("acme").and_then(|v| v.as_table_mut()) {
                if let Some(bin) = &req.acme_bin {
                    acme.insert("bin".to_string(), toml::Value::String(bin.clone()));
                }
                if let Some(home) = &req.acme_home {
                    acme.insert("home".to_string(), toml::Value::String(home.clone()));
                }
            }
            if let Ok(new_content) = toml::to_string_pretty(&config) {
                let _ = std::fs::write(&config_path, new_content);
            }
        }
    }

    Json(json!(ApiResponse::success("设置已保存")))
}

/// 获取主机名
fn get_hostname() -> anyhow::Result<String> {
    let output = cmd::silent_command("hostname").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 获取 Nginx 版本
async fn get_nginx_version(nginx_bin: &str) -> String {
    if nginx_bin.is_empty() {
        return "not installed".to_string();
    }
    let output = cmd::silent_tokio_command(nginx_bin)
        .arg("-v")
        .output()
        .await;

    match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            stderr.lines()
                .find(|l| l.contains("version"))
                .map(|l| l.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        }
        Err(_) => "not installed".to_string(),
    }
}
