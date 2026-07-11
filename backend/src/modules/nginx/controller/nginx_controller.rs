use axum::{extract::State, Extension, Json};
use serde_json::json;

use crate::modules::common::audit::context::SharedAuditContext;
use crate::modules::common::dto::{ApiResponse, NginxTestResult};
use crate::modules::common::nginx::get_nginx_config;
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

#[audit_log(module = "nginx", action = "一键安装Nginx")]
pub async fn install(
    ctx: Extension<SharedAuditContext>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let _ = ctx;
    // 解压到 server/nginx/ 目录
    let install_dir = crate::modules::common::config::get_run_dir()
        .join("server")
        .join("nginx")
        .to_string_lossy()
        .to_string();
    match crate::modules::common::nginx::install_nginx(&install_dir).await {
        Ok(_) => { crate::modules::dashboard::controller::dashboard_ws::trigger_push(&state).await; Json(json!(ApiResponse::success("Nginx安装成功"))) }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("安装失败: {}", e)))),
    }
}
