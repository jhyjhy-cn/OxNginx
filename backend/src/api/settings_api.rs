use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::json;

use crate::dto::ApiResponse;
use crate::AppState;

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
}

#[derive(Debug, serde::Serialize)]
pub struct AcmeSettings {
    pub bin: String,
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
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub nginx_bin: Option<String>,
    pub nginx_config: Option<String>,
    pub nginx_sites_enabled: Option<String>,
    pub acme_bin: Option<String>,
}

/// 获取系统设置
pub async fn get_settings(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    // 获取系统信息
    let hostname = get_hostname().unwrap_or_else(|_| "unknown".to_string());
    let nginx_version = get_nginx_version(&config.nginx.bin).await;

    let settings = SettingsResponse {
        server: ServerSettings {
            host: config.server.host.clone(),
            port: config.server.port,
        },
        nginx: NginxSettings {
            bin: config.nginx.bin.clone(),
            config: config.nginx.config.clone(),
            sites_enabled: config.nginx.sites_enabled.clone(),
        },
        acme: AcmeSettings {
            bin: config.acme.bin.clone(),
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

/// 更新系统设置
pub async fn update_settings(
    State(state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Json<serde_json::Value> {
    // 读取现有配置
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".into());
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置文件失败: {}", e)))),
    };

    let mut config: toml::Value = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("解析配置文件失败: {}", e)))),
    };

    // 更新配置
    if let Some(nginx_bin) = &req.nginx_bin {
        if let Some(nginx) = config.get_mut("nginx") {
            if let Some(table) = nginx.as_table_mut() {
                table.insert("bin".to_string(), toml::Value::String(nginx_bin.clone()));
            }
        }
    }

    if let Some(nginx_config) = &req.nginx_config {
        if let Some(nginx) = config.get_mut("nginx") {
            if let Some(table) = nginx.as_table_mut() {
                table.insert("config".to_string(), toml::Value::String(nginx_config.clone()));
            }
        }
    }

    if let Some(sites_enabled) = &req.nginx_sites_enabled {
        if let Some(nginx) = config.get_mut("nginx") {
            if let Some(table) = nginx.as_table_mut() {
                table.insert("sites_enabled".to_string(), toml::Value::String(sites_enabled.clone()));
            }
        }
    }

    if let Some(acme_bin) = &req.acme_bin {
        if let Some(acme) = config.get_mut("acme") {
            if let Some(table) = acme.as_table_mut() {
                table.insert("bin".to_string(), toml::Value::String(acme_bin.clone()));
            }
        }
    }

    // 保存配置
    let new_content = toml::to_string_pretty(&config).unwrap();
    match std::fs::write(&config_path, new_content) {
        Ok(_) => {
            // 重新加载配置到内存
            match crate::config::AppConfig::load() {
                Ok(new_config) => {
                    state.update_config(new_config);
                    tracing::info!("内存配置已更新");
                }
                Err(e) => {
                    tracing::error!("重新加载配置失败: {}", e);
                }
            }
            Json(json!(ApiResponse::success("设置已保存")))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("保存配置文件失败: {}", e)))),
    }
}

/// 获取主机名
fn get_hostname() -> anyhow::Result<String> {
    let output = std::process::Command::new("hostname").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 获取 Nginx 版本
async fn get_nginx_version(nginx_bin: &str) -> String {
    let output = tokio::process::Command::new(nginx_bin)
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
