use axum::Extension;
use crate::modules::common::audit::context::SharedAuditContext;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::ApiResponse;
use crate::modules::common::nginx::get_nginx_config;
use crate::modules::site::service::site_service;
use crate::AppState;

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
