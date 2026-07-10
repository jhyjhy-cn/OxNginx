use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::sys::dao::param_dao;
use crate::modules::sys::entity::param::Param;

pub async fn page_params(
    pool: &SqlitePool,
    keyword: Option<&str>,
    group_code: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<Param>, i64)> {
    Ok(param_dao::page_params(pool, keyword, group_code, page, page_size).await?)
}

pub async fn get_param(pool: &SqlitePool, id: i64) -> Result<Option<Param>> {
    Ok(param_dao::find_param_by_id(pool, id).await?)
}

pub async fn get_param_by_key(pool: &SqlitePool, key: &str) -> Result<Option<Param>> {
    Ok(param_dao::find_param_by_key(pool, key).await?)
}

pub async fn create_param(
    pool: &SqlitePool,
    key: &str,
    value: Option<&str>,
    name: &str,
    group_code: Option<&str>,
    remark: Option<&str>,
    sort: i32,
) -> Result<i64> {
    // 唯一性校验：key 已存在则报错
    if param_dao::find_param_by_key(pool, key).await?.is_some() {
        anyhow::bail!("参数键已存在");
    }
    Ok(param_dao::insert_param_returning_id(
        pool,
        key,
        value,
        name,
        group_code.unwrap_or("default"),
        remark,
        sort,
    )
    .await?)
}

pub async fn update_param(
    pool: &SqlitePool,
    id: i64,
    value: Option<&str>,
    name: Option<&str>,
    group_code: Option<&str>,
    remark: Option<&str>,
    sort: Option<i32>,
) -> Result<()> {
    Ok(param_dao::update_param_fields(pool, id, value, name, group_code, remark, sort).await?)
}

pub async fn delete_param(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(param_dao::delete_param(pool, id).await? > 0)
}

/// 启动时确保默认参数存在（已存在则跳过，保留管理员在线改的值）
pub async fn ensure_default_params(pool: &SqlitePool) -> Result<()> {
    let defaults = [
        (
            "sys.file.storage.base_path",
            "/files/",
            "文件存储根路径",
            "file",
            "上传文件保存的相对路径（相对于运行目录）",
            0,
        ),
        (
            "sys.file.upload.max_size_mb",
            "10",
            "上传文件大小上限(MB)",
            "file",
            "单文件最大字节数（MB），超过则拒绝",
            1,
        ),
        (
            "sys.file.base_url",
            "http://localhost:9000",
            "文件访问 BaseURL",
            "file",
            "拼接给前端预览/下载的完整 URL 前缀（含协议+端口，不含路径）",
            2,
        ),
    ];
    for (k, v, n, gc, r, s) in defaults {
        // 只在 key 不存在时插入
        if param_dao::find_param_by_key(pool, k).await?.is_none() {
            let _ = param_dao::insert_param_returning_id(pool, k, Some(v), n, gc, Some(r), s).await?;
        }
    }
    Ok(())
}