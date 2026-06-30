use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use crate::dto::ApiResponse;
use crate::service::backup_service;
use crate::AppState;

/// 获取站点备份列表
pub async fn list_backups(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
) -> Json<serde_json::Value> {
    match backup_service::get_backups(&state, site_id).await {
        Ok(backups) => Json(json!(ApiResponse::success(backups))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取备份列表失败: {}", e)))),
    }
}

/// 创建备份
pub async fn create_backup(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
) -> Json<serde_json::Value> {
    // 获取站点当前配置
    let site = match crate::service::site_service::get_site(&state, site_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    };

    let config = crate::nginx::generate_site_config(&site);
    match backup_service::create_backup(&state, site_id, &config).await {
        Ok(backup) => Json(json!(ApiResponse::success(backup))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建备份失败: {}", e)))),
    }
}

/// 恢复备份
pub async fn restore_backup(
    State(state): State<AppState>,
    Path(backup_id): Path<i64>,
) -> Json<serde_json::Value> {
    match backup_service::restore_backup(&state, backup_id).await {
        Ok(Some(backup)) => {
            // 恢复配置文件
            if let Some(site_id) = backup.site_id {
                if let Ok(Some(site)) = crate::service::site_service::get_site(&state, site_id).await {
                    let sites_enabled = &state.config.nginx.sites_enabled;
                    let _ = crate::nginx::write_site_config(sites_enabled, &site.name, &backup.config).await;
                }
            }
            Json(json!(ApiResponse::success(backup)))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("备份不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("恢复备份失败: {}", e)))),
    }
}
