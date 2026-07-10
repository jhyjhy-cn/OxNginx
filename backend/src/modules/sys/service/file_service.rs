use anyhow::{anyhow, Result};
use axum::extract::Multipart;
use chrono::{Datelike, Local};
use md5::Context;
use sqlx::SqlitePool;
use std::path::PathBuf;
use uuid::Uuid;

use crate::modules::common::config::get_run_dir;
use crate::modules::common::dto::UploadFileResponse;
use crate::modules::sys::dao::{file_dao, param_dao};

const DEFAULT_BASE_PATH: &str = "/files/";
const DEFAULT_MAX_SIZE_MB: i64 = 10;

/// 读 base_url 参数，无则返回 None
async fn get_base_url(pool: &SqlitePool) -> Option<String> {
    param_dao::find_param_by_key(pool, "sys.file.base_url")
        .await
        .ok()
        .flatten()
        .and_then(|p| p.value)
        .filter(|s| !s.is_empty())
}

/// base_url + path 拼接；base_url 空时退化为 path 自身
fn build_url(base_url: &Option<String>, path: &str) -> String {
    match base_url {
        Some(base) => format!("{}{}", base.trim_end_matches('/'), path),
        None => path.to_string(),
    }
}

/// 解析 multipart → 落盘 → 写 DB → 返回上传结果
pub async fn upload(
    pool: &SqlitePool,
    mut multipart: Multipart,
    user_id: Option<i64>,
) -> Result<UploadFileResponse> {
    // 读 base_path / base_url / max_size
    let base_path = param_dao::find_param_by_key(pool, "sys.file.storage.base_path")
        .await
        .ok()
        .flatten()
        .and_then(|p| p.value)
        .unwrap_or_else(|| DEFAULT_BASE_PATH.to_string());

    let base_url = param_dao::find_param_by_key(pool, "sys.file.base_url")
        .await
        .ok()
        .flatten()
        .and_then(|p| p.value)
        .filter(|s| !s.is_empty());

    let max_size_mb: i64 = param_dao::find_param_by_key(pool, "sys.file.upload.max_size_mb")
        .await
        .ok()
        .flatten()
        .and_then(|p| p.value)
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_SIZE_MB);
    let max_bytes = max_size_mb * 1024 * 1024;

    // axum 0.8 默认就支持 stream，单次 multipart 第一个字段就是文件
    let field = multipart
        .next_field()
        .await
        .map_err(|e| anyhow!("解析上传字段失败: {}", e))?
        .ok_or_else(|| anyhow!("未找到上传文件"))?;

    let original_name = field
        .file_name()
        .ok_or_else(|| anyhow!("未提供文件名"))?
        .to_string();

    let content_type = field.content_type().map(|s| s.to_string());

    // 一次性读取整个 field 的字节（10 MB 硬上限由下面 size > max_bytes 判定）
    let bytes = field
        .bytes()
        .await
        .map_err(|e| anyhow!("读取上传字节失败: {}", e))?;
    let size = bytes.len() as i64;
    if size > max_bytes {
        return Err(anyhow!("文件超过 {} MB 上限", max_size_mb));
    }
    // md5
    let mut hasher = Context::new();
    hasher.consume(&bytes);
    let md5_hex = hex::encode(hasher.compute().0);

    // 后缀
    let suffix = std::path::Path::new(&original_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 服务器文件名
    let uuid = Uuid::new_v4();
    let server_name = if suffix.is_empty() {
        uuid.to_string()
    } else {
        format!("{}.{}", uuid, suffix)
    };

    // 日期分目录：yyyy/MM/dd
    let now = Local::now();
    let date_path = format!("{:04}/{:02}/{:02}", now.year(), now.month(), now.day());

    // 路径：{run_dir}{base_path}{date_path}/{server_name}
    // base_path 已含前导 /，run_dir 不含尾 /
    let run_dir = get_run_dir();
    let base_trim = base_path.trim_start_matches('/');
    let rel_path = format!("{}/{}/{}", base_trim.trim_end_matches('/'), date_path, server_name);
    let abs_path: PathBuf = run_dir.join(&rel_path);

    if let Some(parent) = abs_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&abs_path, &bytes).await?;

    // 落盘的 path（DB 存 /static/files/yyyy/MM/dd/xxx.ext）
    let stored_path = format!("/{}", rel_path);

    // 写 DB
    let id = file_dao::insert_file_returning_id(
        pool,
        &server_name,
        &original_name,
        &suffix,
        size,
        content_type.as_deref(),
        Some(&md5_hex),
        &stored_path,
        "local",
        None, // dept_id：暂不绑定
        user_id,
    )
    .await?;

    // URL：拼接 base_url + stored_path（base_url 为空时退化为 stored_path，让前端自行处理）
    let url = match base_url {
        Some(base) => format!("{}{}", base.trim_end_matches('/'), stored_path),
        None => stored_path.clone(),
    };

    Ok(UploadFileResponse {
        id,
        name: server_name,
        original_name,
        path: stored_path,
        url,
        size,
        mime_type: content_type,
    })
}

pub async fn page_files(
    pool: &SqlitePool,
    keyword: Option<&str>,
    suffix: Option<&str>,
    provider: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<crate::modules::sys::entity::file::File>, i64)> {
    let base_url = get_base_url(pool).await;
    let (mut list, total) = file_dao::page_files(pool, keyword, suffix, provider, page, page_size).await?;
    for f in &mut list {
        f.url = Some(build_url(&base_url, &f.path));
    }
    Ok((list, total))
}

pub async fn get_file(pool: &SqlitePool, id: i64) -> Result<Option<crate::modules::sys::entity::file::File>> {
    let base_url = get_base_url(pool).await;
    let mut f = file_dao::find_file_by_id(pool, id).await?;
    if let Some(ref mut file) = f {
        file.url = Some(build_url(&base_url, &file.path));
    }
    Ok(f)
}

pub async fn delete_file(pool: &SqlitePool, id: i64) -> Result<bool> {
    // 先拿路径，删完 DB 再删磁盘
    let f = file_dao::find_file_by_id(pool, id).await?;
    let removed = file_dao::delete_file(pool, id).await? > 0;
    if removed {
        if let Some(file) = f {
            let run_dir = get_run_dir();
            let rel = file.path.trim_start_matches('/');
            let abs = run_dir.join(rel);
            let _ = tokio::fs::remove_file(&abs).await;
        }
    }
    Ok(removed)
}

pub async fn delete_files(pool: &SqlitePool, ids: Vec<i64>) -> Result<u64> {
    let rows = file_dao::delete_files(pool, &ids).await?;
    let run_dir = get_run_dir();
    for f in rows {
        let rel = f.path.trim_start_matches('/');
        let abs = run_dir.join(rel);
        let _ = tokio::fs::remove_file(&abs).await;
    }
    Ok(ids.len() as u64)
}