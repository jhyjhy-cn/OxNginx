use axum::{extract::State, Extension, Json};
use serde_json::json;
use sqlx::SqlitePool;

use crate::modules::common::audit::context::SharedAuditContext;
use crate::modules::common::dto::{ApiResponse, NginxTestResult};
use crate::modules::common::nginx::{get_nginx_config, NginxInstallResult};
use crate::modules::sys::service::param_service;
use crate::AppState;
use ox_nginx_macros::audit_log;

pub async fn test_config(State(state): State<AppState>) -> Json<serde_json::Value> {
    let nginx_bin = match get_nginx_config(&state).await {
        Ok(cfg) => match cfg.bin {
            Some(bin) => bin,
            None => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
        },
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    Json(json!(ApiResponse::success(crate::modules::common::nginx::test_config(&nginx_bin).await)))
}

#[audit_log(module = "nginx", action = "重载Nginx配置")]
pub async fn reload(
    ctx: Extension<SharedAuditContext>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let _ = ctx; // ponytail: 宏已通过 ctx 写入 module/action，参数声明是宏工作的前提
    let nginx_bin = match get_nginx_config(&state).await {
        Ok(cfg) => match cfg.bin {
            Some(bin) => bin,
            None => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
        },
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    let test_result = crate::modules::common::nginx::test_config(&nginx_bin).await;
    if !test_result.success {
        return Json(json!(ApiResponse::<()>::error(format!("配置测试失败，禁止重载: {}", test_result.message))));
    }
    let result = match crate::modules::common::nginx::reload_nginx(&nginx_bin).await {
        Ok(true) => Ok(NginxTestResult { success: true, message: "Nginx重载成功".into() }),
        Ok(false) => Err("Nginx重载失败".to_string()),
        Err(e) => Err(format!("Nginx重载失败: {}", e)),
    };
    crate::modules::dashboard::controller::dashboard_ws::trigger_push(&state).await;
    match result {
        Ok(r) => Json(json!(ApiResponse::success(r))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e))),
    }
}

pub async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let nginx_bin = match get_nginx_config(&state).await {
        Ok(cfg) => match cfg.bin {
            Some(bin) => bin,
            None => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
        },
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    Json(json!(ApiResponse::success(crate::modules::common::nginx::get_nginx_status(&nginx_bin).await)))
}

#[audit_log(module = "nginx", action = "启动Nginx")]
pub async fn start(
    ctx: Extension<SharedAuditContext>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let _ = ctx;
    let (nginx_bin, nginx_config) = match get_nginx_config(&state).await {
        Ok(cfg) => {
            let bin = match cfg.bin {
                Some(b) => b,
                None => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
            };
            let config = match cfg.config {
                Some(c) => c,
                None => return Json(json!(ApiResponse::<()>::error("Nginx配置不完整，请检查系统参数"))),
            };
            (bin, config)
        },
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    match crate::modules::common::nginx::start_nginx(&nginx_bin, &nginx_config).await {
        Ok(true) => { crate::modules::dashboard::controller::dashboard_ws::trigger_push(&state).await; Json(json!(ApiResponse::success("Nginx已启动"))) }
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx启动失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx启动失败: {}", e)))),
    }
}

#[audit_log(module = "nginx", action = "停止Nginx")]
pub async fn stop(
    ctx: Extension<SharedAuditContext>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let _ = ctx;
    let nginx_bin = match get_nginx_config(&state).await {
        Ok(cfg) => match cfg.bin {
            Some(bin) => bin,
            None => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
        },
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    match crate::modules::common::nginx::stop_nginx(&nginx_bin).await {
        Ok(true) => { crate::modules::dashboard::controller::dashboard_ws::trigger_push(&state).await; Json(json!(ApiResponse::success("Nginx已停止"))) }
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx停止失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx停止失败: {}", e)))),
    }
}

#[audit_log(module = "nginx", action = "重启Nginx")]
pub async fn restart(
    ctx: Extension<SharedAuditContext>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let _ = ctx;
    let (nginx_bin, nginx_config) = match get_nginx_config(&state).await {
        Ok(cfg) => {
            let bin = match cfg.bin {
                Some(b) => b,
                None => return Json(json!(ApiResponse::<()>::error("Nginx未安装，请先执行一键安装"))),
            };
            let config = match cfg.config {
                Some(c) => c,
                None => return Json(json!(ApiResponse::<()>::error("Nginx配置不完整，请检查系统参数"))),
            };
            (bin, config)
        },
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("读取配置失败: {}", e)))),
    };
    match crate::modules::common::nginx::restart_nginx(&nginx_bin, &nginx_config).await {
        Ok(true) => { crate::modules::dashboard::controller::dashboard_ws::trigger_push(&state).await; Json(json!(ApiResponse::success("Nginx已重启"))) }
        Ok(false) => Json(json!(ApiResponse::<()>::error("Nginx重启失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("Nginx重启失败: {}", e)))),
    }
}

/// 将 nginx 安装路径写入 sys_params
async fn write_nginx_params_to_db(pool: &SqlitePool, result: &NginxInstallResult) -> anyhow::Result<()> {
    let exe_dir = crate::modules::common::config::get_run_dir();

    let ssl_dir = exe_dir.join("ssl").to_string_lossy().replace('\\', "/");
    let default_root = exe_dir.join("wwwroot").to_string_lossy().replace('\\', "/");
    let log_access = exe_dir
        .join("wwwlogs")
        .join("nginx")
        .join("access.log")
        .to_string_lossy()
        .replace('\\', "/");
    let log_error = exe_dir
        .join("wwwlogs")
        .join("nginx")
        .join("error.log")
        .to_string_lossy()
        .replace('\\', "/");

    let params = [
        ("nginx.bin", result.bin.replace('\\', "/")),
        ("nginx.config", result.config.replace('\\', "/")),
        ("nginx.sites_enabled", result.sites_enabled.replace('\\', "/")),
        ("nginx.ssl_dir", ssl_dir),
        ("nginx.default_root", default_root),
        ("nginx.log_access", log_access),
        ("nginx.log_error", log_error),
    ];

    for (key, value) in params {
        let param = param_service::get_param_by_key(pool, key).await?;
        if let Some(p) = param {
            param_service::update_param(pool, p.id, Some(&value), None, None, None, None).await?;
        }
    }
    Ok(())
}

#[audit_log(module = "nginx", action = "一键安装Nginx")]
pub async fn install(
    ctx: Extension<SharedAuditContext>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let _ = ctx;
    let install_dir = crate::modules::common::config::get_run_dir()
        .join("server")
        .join("nginx")
        .to_string_lossy()
        .to_string();

    match crate::modules::common::nginx::install_nginx(&install_dir).await {
        Ok(result) => {
            if let Err(e) = write_nginx_params_to_db(state.db.pool(), &result).await {
                tracing::error!("写入nginx系统参数失败: {}", e);
            }
            crate::modules::dashboard::controller::dashboard_ws::trigger_push(&state).await;
            Json(json!(ApiResponse::success("Nginx安装成功")))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("安装失败: {}", e)))),
    }
}
