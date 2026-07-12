use axum::{
    body::Body,
    extract::{Extension, Multipart, Path, Query, State},
    Json,
};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::Response;
use serde::Deserialize;
use serde_json::json;
use tokio_util::io::ReaderStream;

use ox_nginx_macros::check_permission;

use crate::modules::common::config::get_run_dir;
use crate::modules::common::dto::{self, ApiResponse, BatchDeleteFilesRequest};
use crate::modules::common::middleware::TokenInfo;
use crate::modules::sys::service::file_service as svc;
use crate::AppState;

// ============== 系统文件 =============

#[derive(Debug, Deserialize)]
pub struct PageFilesQuery {
    #[serde(default, deserialize_with = "dto::empty_str_opt")]
    pub keyword: Option<String>,
    #[serde(default, deserialize_with = "dto::empty_str_opt")]
    pub suffix: Option<String>,
    #[serde(default, deserialize_with = "dto::empty_str_opt")]
    pub provider: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[check_permission("sys:file:upload")]
pub async fn upload_file(
    State(state): State<AppState>,
    Extension(token): Extension<TokenInfo>,
    multipart: Multipart,
) -> Json<serde_json::Value> {
    match svc::upload(&state.db.pool(), multipart, Some(token.user_id)).await {
        Ok(r) => Json(json!(ApiResponse::success(r))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:file:query")]
pub async fn page_files(
    State(state): State<AppState>,
    Query(q): Query<PageFilesQuery>,
    token: Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    match svc::page_files(
        &state.db.pool(),
        q.keyword.as_deref(),
        q.suffix.as_deref(),
        q.provider.as_deref(),
        page,
        page_size,
    )
    .await
    {
        Ok((list, total)) => Json(json!(ApiResponse::success(serde_json::json!({
            "list": list,
            "total": total,
            "page": page,
            "page_size": page_size,
        })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:file:query")]
pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    token: Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    match svc::get_file(&state.db.pool(), id).await {
        Ok(Some(f)) => Json(json!(ApiResponse::success(f))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("文件不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:file:delete")]
pub async fn delete_file(
    State(state): State<AppState>,
    token: Extension<TokenInfo>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match svc::delete_file(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("文件不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:file:batchDelete")]
pub async fn batch_delete_files(
    State(state): State<AppState>,
    token: Extension<TokenInfo>,
    Json(req): Json<BatchDeleteFilesRequest>,
) -> Json<serde_json::Value> {
    if req.ids.is_empty() {
        return Json(json!(ApiResponse::<()>::error("请选择要删除的文件")));
    }
    match svc::delete_files(&state.db.pool(), req.ids).await {
        Ok(n) => Json(json!(ApiResponse::success(n))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn download_file(
    State(state): State<AppState>,
    _token: Extension<TokenInfo>,
    Path(id): Path<i64>,
) -> Result<Response, (StatusCode, String)> {
    let f = svc::get_file(&state.db.pool(), id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "文件不存在".to_string()))?;

    let run_dir = get_run_dir();
    let rel = f.path.trim_start_matches('/');
    let abs = run_dir.join(rel);
    let file = tokio::fs::File::open(&abs)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("打开文件失败: {}", e)))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 简单的 ASCII filename 头；中文走 RFC 5987 比较麻烦，浏览器 fallback 用 path 里的 uuid 文件名
    let disposition = format!(
        "attachment; filename=\"{}\"",
        sanitize_ascii_filename(&f.original_name)
    );

    let mut builder = Response::builder().status(StatusCode::OK);
    if let Ok(v) = HeaderValue::from_str(&disposition) {
        builder = builder.header(header::CONTENT_DISPOSITION, v);
    } else {
        builder = builder.header(header::CONTENT_DISPOSITION, "attachment");
    }
    if let Some(mime) = &f.mime_type {
        if let Ok(v) = HeaderValue::from_str(mime) {
            builder = builder.header(header::CONTENT_TYPE, v);
        }
    }
    builder = builder.header(header::CONTENT_LENGTH, f.size as u64);

    Ok(builder.body(body).unwrap())
}

/// 提取 ASCII 部分做 filename，避免 header 非法字符
fn sanitize_ascii_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect::<String>()
        .if_empty_then("download")
}

/// ponytail: 字符串兜底
trait IfEmpty {
    fn if_empty_then(self, fallback: &str) -> String;
}
impl IfEmpty for String {
    fn if_empty_then(self, fallback: &str) -> String {
        if self.is_empty() { fallback.into() } else { self }
    }
}