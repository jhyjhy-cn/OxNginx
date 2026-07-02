use axum::{extract::State, Json};
use serde_json::json;

use crate::dto::{ApiResponse, NginxTestResult};
use crate::AppState;

/// 测试Nginx配置
pub async fn test_config(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let result = crate::nginx::test_config(&config.nginx.bin).await;
    Json(json!(ApiResponse::success(result)))
}

/// 重载Nginx配置
pub async fn reload(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    // 先测试配置
    let test_result = crate::nginx::test_config(&config.nginx.bin).await;
    if !test_result.success {
        return Json(json!(ApiResponse::<()>::error(format!(
            "配置测试失败，禁止重载: {}",
            test_result.message
        ))));
    }

    // 重载
    match crate::nginx::reload_nginx(&config.nginx.bin).await {
        Ok(true) => Json(json!(ApiResponse::success(NginxTestResult {
            success: true,
            message: "Nginx重载成功".into(),
        }))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx重载失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx重载失败: {}", e)))),
    }
}

/// 获取Nginx状态
pub async fn status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    let status = crate::nginx::get_nginx_status(&config.nginx.bin).await;
    Json(json!(ApiResponse::success(status)))
}

/// 启动Nginx
pub async fn start(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    match crate::nginx::start_nginx(&config.nginx.bin).await {
        Ok(true) => Json(json!(ApiResponse::success(NginxTestResult {
            success: true,
            message: "Nginx启动成功".into(),
        }))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx启动失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx启动失败: {}", e)))),
    }
}

/// 停止Nginx
pub async fn stop(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    match crate::nginx::stop_nginx(&config.nginx.bin).await {
        Ok(true) => Json(json!(ApiResponse::success(NginxTestResult {
            success: true,
            message: "Nginx已停止".into(),
        }))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx停止失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx停止失败: {}", e)))),
    }
}

/// 重启Nginx
pub async fn restart(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let config = state.get_config();
    // 先测试配置
    let test_result = crate::nginx::test_config(&config.nginx.bin).await;
    if !test_result.success {
        return Json(json!(ApiResponse::<()>::error(format!(
            "配置测试失败，禁止重启: {}",
            test_result.message
        ))));
    }

    match crate::nginx::restart_nginx(&config.nginx.bin).await {
        Ok(true) => Json(json!(ApiResponse::success(NginxTestResult {
            success: true,
            message: "Nginx重启成功".into(),
        }))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx重启失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx重启失败: {}", e)))),
    }
}

/// 一键安装Nginx
pub async fn install(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    use std::env::consts::OS;
    use std::path::PathBuf;

    // 确定安装目录
    let install_dir = if OS == "windows" {
        std::env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join("nginx").to_string_lossy().to_string())
            .unwrap_or_else(|_| "C:\\nginx".to_string())
    } else {
        "/opt/oxnginx/server/nginx".to_string()
    };

    tracing::info!("Nginx 安装目录: {}", install_dir);

    match crate::nginx::install_nginx(&install_dir).await {
        Ok(result) => {
            tracing::info!("Nginx 安装成功: {}", result.bin);

            // 更新配置文件
            let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".into());
            let mut config_content = match std::fs::read_to_string(&config_path) {
                Ok(c) => c,
                Err(e) => {
                    return Json(json!(ApiResponse::<()>::error(format!("读取配置文件失败: {}", e))));
                }
            };

            // 获取旧配置用于替换（使用 get_config 避免跨 await 持有锁）
            let old_config = state.get_config();

            // 替换 nginx 配置（TOML 中路径使用正斜杠）
            let new_bin = result.bin.replace('\\', "/");
            let new_config = result.config.replace('\\', "/");
            let new_sites = result.sites_enabled.replace('\\', "/");
            config_content = config_content
                .replace(&old_config.nginx.bin, &new_bin)
                .replace(&old_config.nginx.config, &new_config)
                .replace(&old_config.nginx.sites_enabled, &new_sites);

            if let Err(e) = std::fs::write(&config_path, &config_content) {
                return Json(json!(ApiResponse::<()>::error(format!("写入配置文件失败: {}", e))));
            }

            tracing::info!("配置文件已更新: {}", config_path);

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

            Json(json!(ApiResponse::success(serde_json::json!({
                "message": "Nginx 安装成功",
                "bin": result.bin,
                "config": result.config,
                "sites_enabled": result.sites_enabled
            }))))
        }
        Err(e) => {
            tracing::error!("Nginx 安装失败: {}", e);
            Json(json!(ApiResponse::<()>::error(format!("Nginx 安装失败: {}", e))))
        }
    }
}
