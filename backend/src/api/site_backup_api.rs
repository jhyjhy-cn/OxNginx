use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::dto::ApiResponse;
use crate::service::{site_service, site_backup_service};
use crate::AppState;
use ox_nginx_macros::operation_log;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 { 1 }
fn default_page_size() -> u32 { 20 }

/// 批量删除请求
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub filenames: Vec<String>,
}

/// 获取站点备份列表（分页）
pub async fn list_site_backups(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
    Query(pq): Query<PageQuery>,
) -> Json<serde_json::Value> {
    let site = match site_service::get_site(&state, site_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    };

    match site_backup_service::list_backups(&site.name, pq.page, pq.page_size) {
        Ok(page) => Json(json!(ApiResponse::success(page))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取备份列表失败: {}", e)))),
    }
}

/// 创建站点备份
#[operation_log("创建站点备份")]
pub async fn create_site_backup(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
) -> Json<serde_json::Value> {
    let site = match site_service::get_site(&state, site_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    };

    let root_path = match &site.root_path {
        Some(p) if !p.is_empty() => p.as_str(),
        _ => return Json(json!(ApiResponse::<()>::error("站点未配置根目录"))),
    };

    match site_backup_service::create_backup(&site.name, root_path) {
        Ok(info) => Json(json!(ApiResponse::success(info))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建备份失败: {}", e)))),
    }
}

/// 下载站点备份
pub async fn download_site_backup(
    State(state): State<AppState>,
    Path((site_id, filename)): Path<(i64, String)>,
) -> impl IntoResponse {
    let site = match site_service::get_site(&state, site_id).await {
        Ok(Some(s)) => s,
        _ => return Json(json!(ApiResponse::<()>::error("站点不存在"))).into_response(),
    };

    let path = match site_backup_service::get_backup_path(&site.name, &filename) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e.to_string()))).into_response(),
    };

    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let headers = [
                ("Content-Type", "application/zip".to_string()),
                ("Content-Disposition", format!("attachment; filename=\"{}\"", filename)),
            ];
            (axum::http::HeaderMap::from_iter(
                headers.iter().map(|(k, v)| (
                    k.parse::<axum::http::header::HeaderName>().unwrap(),
                    v.parse().unwrap(),
                )),
            ), bytes)
                .into_response()
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取文件失败: {}", e)))).into_response(),
    }
}

/// 删除站点备份
#[operation_log("删除站点备份")]
pub async fn delete_site_backup(
    State(state): State<AppState>,
    Path((site_id, filename)): Path<(i64, String)>,
) -> Json<serde_json::Value> {
    let site = match site_service::get_site(&state, site_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    };

    match site_backup_service::delete_backup(&site.name, &filename) {
        Ok(()) => Json(json!(ApiResponse::success("备份已删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除备份失败: {}", e)))),
    }
}

/// 批量删除站点备份
#[operation_log("批量删除备份")]
pub async fn batch_delete_site_backups(
    State(state): State<AppState>,
    Path(site_id): Path<i64>,
    Json(req): Json<BatchDeleteRequest>,
) -> Json<serde_json::Value> {
    let site = match site_service::get_site(&state, site_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("站点不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("获取站点失败: {}", e)))),
    };

    let mut success = 0;
    let mut errors = Vec::new();
    for filename in &req.filenames {
        match site_backup_service::delete_backup(&site.name, filename) {
            Ok(()) => success += 1,
            Err(e) => errors.push(format!("{}: {}", filename, e)),
        }
    }

    if errors.is_empty() {
        Json(json!(ApiResponse::success(serde_json::json!({ "deleted": success }))))
    } else {
        Json(json!(ApiResponse::success(serde_json::json!({
            "deleted": success,
            "errors": errors,
        }))))
    }
}
