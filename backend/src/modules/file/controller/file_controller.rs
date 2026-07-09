use axum::extract::Query;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, Extension, Json};
use crate::modules::common::audit::context::SharedAuditContext;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

use crate::modules::common::dto::file_dto::*;
use crate::modules::common::dto::ApiResponse;
use crate::modules::file::service::file_service;
use crate::AppState;
use ox_nginx_macros::audit_log;

/// 查询参数（GET 接口用）
#[derive(Debug, Deserialize, Serialize)]
pub struct PathQuery {
    pub path: String,
}

/// POST 请求参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ListFilesRequest {
    pub path: String,
    #[serde(default)]
    pub search: String,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 100 }

/// 列出目录内容
pub async fn list_files(
    State(state): State<AppState>,
    Json(req): Json<ListFilesRequest>,
) -> Json<serde_json::Value> {
    let path = if req.path.is_empty() {
        #[cfg(target_os = "windows")]
        {
            "C:\\".to_string()
        }
        #[cfg(not(target_os = "windows"))]
        {
            "/".to_string()
        }
    } else {
        req.path
    };

    // 如果请求的是根级别，返回根目录列表
    #[cfg(target_os = "windows")]
    if path == "/" || path == "\\" {
        let roots = file_service::get_root_dirs();
        let items: Vec<FileItem> = roots
            .into_iter()
            .map(|drive| FileItem {
                name: drive.clone(),
                path: drive,
                is_dir: true,
                size: 0,
                permissions: String::new(),
                owner: "SYSTEM".to_string(),
                modified: String::new(),
                extension: String::new(),
                note: None,
            })
            .collect();
        return Json(json!(ApiResponse::success(FileListResponse {
            path: "/".to_string(),
            parent: None,
            total: items.len(),
            dir_count: items.iter().filter(|i| i.is_dir).count(),
            file_count: items.iter().filter(|i| !i.is_dir).count(),
            items,
        })));
    }

    let parent = Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string());

    match file_service::list_directory(&state, &path).await {
        Ok(mut items) => {
            // 批量查询备注
            let paths: Vec<String> = items.iter().map(|i| i.path.clone()).collect();
            match file_service::get_notes(&state, &paths).await {
                Ok(notes) => {
                    let note_map: std::collections::HashMap<String, String> =
                        notes.into_iter().collect();
                    for item in &mut items {
                        item.note = note_map.get(&item.path).cloned();
                    }
                }
                Err(e) => {
                    tracing::warn!("查询文件备注失败: {}", e);
                }
            }

            // 搜索过滤
            if !req.search.is_empty() {
                let q = req.search.to_lowercase();
                items.retain(|i| i.name.to_lowercase().contains(&q));
            }

            let total = items.len();
            let dir_count = items.iter().filter(|i| i.is_dir).count();
            let file_count = items.iter().filter(|i| !i.is_dir).count();

            // 分页
            let page = req.page.max(1);
            let page_size = req.page_size.clamp(1, 10000);
            let start = (page - 1) * page_size;
            let paged: Vec<FileItem> = items.into_iter().skip(start).take(page_size).collect();

            Json(json!(ApiResponse::success(FileListResponse {
                path,
                parent,
                items: paged,
                total,
                dir_count,
                file_count,
            })))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("列出目录失败: {}", e)))),
    }
}

/// 获取根目录列表
pub async fn list_roots() -> Json<serde_json::Value> {
    let roots = file_service::get_root_dirs();
    Json(json!(ApiResponse::success(roots)))
}

/// 读取文件内容
pub async fn read_file(Json(req): Json<PathQuery>) -> Json<serde_json::Value> {
    match file_service::read_file(&req.path).await {
        Ok((true, content)) => Json(json!(ApiResponse::success(json!({
            "path": req.path,
            "content": content,
            "is_binary": false,
        })))),
        Ok((false, _)) => Json(json!(ApiResponse::success(json!({
            "path": req.path,
            "content": null,
            "is_binary": true,
        })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取文件失败: {}", e)))),
    }
}

/// 写入文件
#[audit_log(module = "file", action = "写入文件", capture = req)]
pub async fn write_file(
    ctx: Extension<SharedAuditContext>,
    
    State(_state): State<AppState>,
    Json(req): Json<FileWriteRequest>,
) -> Json<serde_json::Value> {
    match file_service::write_file(&req.path, &req.content).await {
        Ok(_) => Json(json!(ApiResponse::success("文件已保存"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("写入文件失败: {}", e)))),
    }
}

/// 创建目录
#[audit_log(module = "file", action = "创建目录", capture = req)]
pub async fn mkdir(
    ctx: Extension<SharedAuditContext>,
    Json(req): Json<FileMkdirRequest>) -> Json<serde_json::Value> {
    match file_service::create_dir(&req.path, &req.name).await {
        Ok(new_path) => Json(json!(ApiResponse::success(json!({ "path": new_path })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建目录失败: {}", e)))),
    }
}

/// 创建文件
#[audit_log(module = "file", action = "创建文件", capture = req)]
pub async fn touch(
    ctx: Extension<SharedAuditContext>,
    Json(req): Json<FileTouchRequest>) -> Json<serde_json::Value> {
    match file_service::create_file(&req.path, &req.name).await {
        Ok(new_path) => Json(json!(ApiResponse::success(json!({ "path": new_path })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建文件失败: {}", e)))),
    }
}

/// 重命名
#[audit_log(module = "file", action = "重命名文件", capture = req)]
pub async fn rename(
    ctx: Extension<SharedAuditContext>,
    
    Json(req): Json<FileRenameRequest>,
) -> Json<serde_json::Value> {
    match file_service::rename(&req.path, &req.new_name).await {
        Ok(new_path) => Json(json!(ApiResponse::success(json!({ "path": new_path })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("重命名失败: {}", e)))),
    }
}

/// 移动文件
#[audit_log(module = "file", action = "移动文件", capture = req)]
pub async fn move_file(
    ctx: Extension<SharedAuditContext>,
    Json(req): Json<FileMoveRequest>) -> Json<serde_json::Value> {
    match file_service::move_path(&req.source, &req.destination).await {
        Ok(_) => Json(json!(ApiResponse::success("移动成功"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("移动失败: {}", e)))),
    }
}

/// 复制文件
#[audit_log(module = "file", action = "复制文件", capture = req)]
pub async fn copy_file(
    ctx: Extension<SharedAuditContext>,
    Json(req): Json<FileCopyRequest>) -> Json<serde_json::Value> {
    match file_service::copy_path(&req.source, &req.destination).await {
        Ok(_) => Json(json!(ApiResponse::success("复制成功"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("复制失败: {}", e)))),
    }
}

/// 删除文件
#[audit_log(module = "file", action = "删除文件", capture = req)]
pub async fn delete_file(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Json(req): Json<FileDeleteRequest>,
) -> Json<serde_json::Value> {
    match file_service::delete_path(&req.path).await {
        Ok(_) => {
            // 同时删除备注
            let _ = file_service::delete_note(&state, &req.path).await;
            Json(json!(ApiResponse::success("删除成功")))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除失败: {}", e)))),
    }
}

/// 修改权限
#[audit_log(module = "file", action = "修改权限", capture = req)]
pub async fn chmod(
    ctx: Extension<SharedAuditContext>,
    Json(req): Json<FileChmodRequest>) -> Json<serde_json::Value> {
    match file_service::chmod(&req.path, &req.mode).await {
        Ok(_) => Json(json!(ApiResponse::success("权限已修改"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("修改权限失败: {}", e)))),
    }
}

/// 压缩文件
#[audit_log(module = "file", action = "压缩文件", capture = req)]
pub async fn compress(
    ctx: Extension<SharedAuditContext>,
    Json(req): Json<FileCompressRequest>) -> Json<serde_json::Value> {
    match file_service::compress(&req.paths, &req.destination, &req.format).await {
        Ok(_) => Json(json!(ApiResponse::success("压缩完成"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("压缩失败: {}", e)))),
    }
}

/// 解压文件
#[audit_log(module = "file", action = "解压文件", capture = req)]
pub async fn extract(
    ctx: Extension<SharedAuditContext>,
    Json(req): Json<FileExtractRequest>) -> Json<serde_json::Value> {
    match file_service::extract(&req.path, &req.destination).await {
        Ok(_) => Json(json!(ApiResponse::success("解压完成"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("解压失败: {}", e)))),
    }
}

/// 保存备注
#[audit_log(module = "file", action = "保存备注", capture = req)]
pub async fn save_note(
    ctx: Extension<SharedAuditContext>,
    
    State(state): State<AppState>,
    Json(req): Json<FileNoteRequest>,
) -> Json<serde_json::Value> {
    match file_service::save_note(&state, &req.path, &req.note).await {
        Ok(_) => Json(json!(ApiResponse::success("备注已保存"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("保存备注失败: {}", e)))),
    }
}

/// 计算文件/目录大小
pub async fn calc_size(Json(req): Json<PathQuery>) -> Json<serde_json::Value> {
    match file_service::calc_size(&req.path).await {
        Ok(size) => Json(json!(ApiResponse::success(json!({ "size": size })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("计算大小失败: {}", e)))),
    }
}

/// 下载文件
pub async fn download_file(Query(query): Query<PathQuery>) -> Response {
    let path = Path::new(&query.path);
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string());

    match tokio::fs::read(&query.path).await {
        Ok(content) => (
            [
                (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", file_name),
                ),
            ],
            content,
        )
            .into_response(),
        Err(e) => {
            let body = Json(json!(ApiResponse::<()>::error(format!("下载失败: {}", e))));
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
        }
    }
}
