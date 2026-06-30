use axum::{extract::State, Json};
use serde_json::json;

use crate::dto::{ApiResponse, NginxTestResult};
use crate::AppState;

/// 测试Nginx配置
pub async fn test_config(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let result = crate::nginx::test_config(&state.config.nginx.bin).await;
    Json(json!(ApiResponse::success(result)))
}

/// 重载Nginx配置
pub async fn reload(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // 先测试配置
    let test_result = crate::nginx::test_config(&state.config.nginx.bin).await;
    if !test_result.success {
        return Json(json!(ApiResponse::<()>::error(format!(
            "配置测试失败，禁止重载: {}",
            test_result.message
        ))));
    }

    // 重载
    match crate::nginx::reload_nginx(&state.config.nginx.bin).await {
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
    let status = crate::nginx::get_nginx_status(&state.config.nginx.bin).await;
    Json(json!(ApiResponse::success(status)))
}

/// 启动Nginx
pub async fn start(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match crate::nginx::start_nginx(&state.config.nginx.bin).await {
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
    match crate::nginx::stop_nginx(&state.config.nginx.bin).await {
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
    // 先测试配置
    let test_result = crate::nginx::test_config(&state.config.nginx.bin).await;
    if !test_result.success {
        return Json(json!(ApiResponse::<()>::error(format!(
            "配置测试失败，禁止重启: {}",
            test_result.message
        ))));
    }

    match crate::nginx::restart_nginx(&state.config.nginx.bin).await {
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
        // Windows: 使用用户目录或 ProgramData
        std::env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join("nginx").to_string_lossy().to_string())
            .unwrap_or_else(|_| "C:\\nginx".to_string())
    } else {
        // Linux: 使用 /opt/nginx
        "/opt/nginx".to_string()
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

            // 替换 nginx 配置
            config_content = config_content
                .replace(&state.config.nginx.bin, &result.bin)
                .replace(&state.config.nginx.config, &result.config)
                .replace(&state.config.nginx.sites_enabled, &result.sites_enabled);

            if let Err(e) = std::fs::write(&config_path, &config_content) {
                return Json(json!(ApiResponse::<()>::error(format!("写入配置文件失败: {}", e))));
            }

            tracing::info!("配置文件已更新: {}", config_path);

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
