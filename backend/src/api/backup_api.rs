use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::dto::ApiResponse;
use crate::service::backup_service;
use crate::AppState;
use ox_nginx_macros::operation_log;

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
#[operation_log("创建备份")]
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
#[operation_log("恢复备份")]
pub async fn restore_backup(
    State(state): State<AppState>,
    Path(backup_id): Path<i64>,
) -> Json<serde_json::Value> {
    match backup_service::restore_backup(&state, backup_id).await {
        Ok(Some(backup)) => {
            // 恢复配置文件
            if let Some(site_id) = backup.site_id {
                if let Ok(Some(site)) = crate::service::site_service::get_site(&state, site_id).await {
                    let config = state.get_config();
                    let sites_enabled = &config.nginx.sites_enabled;
                    let _ = crate::nginx::write_site_config(sites_enabled, &site.name, &backup.config).await;
                }
            }
            Json(json!(ApiResponse::success(backup)))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("备份不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("恢复备份失败: {}", e)))),
    }
}

/// 对比请求
#[derive(Debug, Deserialize)]
pub struct DiffRequest {
    pub id1: i64,
    pub id2: i64,
}

/// 对比两个备份
pub async fn diff_backups(
    State(state): State<AppState>,
    Json(req): Json<DiffRequest>,
) -> Json<serde_json::Value> {
    let backup1 = match backup_service::restore_backup(&state, req.id1).await {
        Ok(Some(b)) => b,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("备份1不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取备份1失败: {}", e)))),
    };

    let backup2 = match backup_service::restore_backup(&state, req.id2).await {
        Ok(Some(b)) => b,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("备份2不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取备份2失败: {}", e)))),
    };

    // 生成差异
    let diff = compute_diff(&backup1.config, &backup2.config);

    Json(json!(ApiResponse::success(serde_json::json!({
        "backup1": {
            "id": backup1.id,
            "version": backup1.version,
            "created_at": backup1.created_at,
        },
        "backup2": {
            "id": backup2.id,
            "version": backup2.version,
            "created_at": backup2.created_at,
        },
        "diff": diff,
    }))))
}

/// 计算差异
fn compute_diff(old: &str, new: &str) -> Vec<serde_json::Value> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut result = Vec::new();

    // 简单的逐行对比
    let max_len = old_lines.len().max(new_lines.len());

    for i in 0..max_len {
        let old_line = old_lines.get(i).copied();
        let new_line = new_lines.get(i).copied();

        match (old_line, new_line) {
            (Some(old), Some(new)) => {
                if old != new {
                    result.push(serde_json::json!({
                        "type": "changed",
                        "line": i + 1,
                        "old": old,
                        "new": new,
                    }));
                } else {
                    result.push(serde_json::json!({
                        "type": "unchanged",
                        "line": i + 1,
                        "content": old,
                    }));
                }
            }
            (Some(old), None) => {
                result.push(serde_json::json!({
                    "type": "deleted",
                    "line": i + 1,
                    "content": old,
                }));
            }
            (None, Some(new)) => {
                result.push(serde_json::json!({
                    "type": "added",
                    "line": i + 1,
                    "content": new,
                }));
            }
            (None, None) => {}
        }
    }

    result
}

/// 删除备份
#[operation_log("删除备份")]
pub async fn delete_backup(
    State(state): State<AppState>,
    Path(backup_id): Path<i64>,
) -> Json<serde_json::Value> {
    match backup_service::delete_backup(&state, backup_id).await {
        Ok(true) => Json(json!(ApiResponse::success("备份已删除"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("删除备份失败"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除备份失败: {}", e)))),
    }
}
