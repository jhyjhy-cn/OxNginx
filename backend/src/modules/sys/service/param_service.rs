use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::sys::dao::param_dao;
use crate::modules::sys::entity::param::Param;

/// Nginx 配置（从数据库 sys_params 读取）
#[derive(Debug, Clone, Default)]
pub struct NginxConfigFromDb {
    pub bin: Option<String>,
    pub config: Option<String>,
    pub sites_enabled: Option<String>,
    pub ssl_dir: Option<String>,
    pub default_root: Option<String>,
    pub log_access: Option<String>,
    pub log_error: Option<String>,
}

/// 从数据库获取 Nginx 配置（如果未设置则返回 None）
pub async fn get_nginx_config(pool: &SqlitePool) -> Result<NginxConfigFromDb> {
    let keys = [
        "nginx.bin",
        "nginx.config",
        "nginx.sites_enabled",
        "nginx.ssl_dir",
        "nginx.default_root",
        "nginx.log_access",
        "nginx.log_error",
    ];
    let mut config = NginxConfigFromDb {
        bin: None,
        config: None,
        sites_enabled: None,
        ssl_dir: None,
        default_root: None,
        log_access: None,
        log_error: None,
    };
    for key in keys {
        if let Some(param) = param_dao::find_param_by_key(pool, key).await? {
            let value = param.value.as_deref().unwrap_or("");
            match key {
                "nginx.bin" if !value.is_empty() => config.bin = Some(value.to_string()),
                "nginx.config" if !value.is_empty() => config.config = Some(value.to_string()),
                "nginx.sites_enabled" if !value.is_empty() => config.sites_enabled = Some(value.to_string()),
                "nginx.ssl_dir" if !value.is_empty() => config.ssl_dir = Some(value.to_string()),
                "nginx.default_root" if !value.is_empty() => config.default_root = Some(value.to_string()),
                "nginx.log_access" if !value.is_empty() => config.log_access = Some(value.to_string()),
                "nginx.log_error" if !value.is_empty() => config.log_error = Some(value.to_string()),
                _ => {}
            }
        }
    }
    Ok(config)
}

/// 确保 nginx 配置参数存在（首次调用时插入，后续跳过）
pub async fn ensure_nginx_params(pool: &SqlitePool) -> Result<()> {
    let nginx_params = [
        ("nginx.bin", "", "Nginx 可执行文件路径", "nginx", "nginx 主程序完整路径", 0),
        ("nginx.config", "", "Nginx 配置文件路径", "nginx", "nginx.conf 完整路径", 1),
        ("nginx.sites_enabled", "", "站点配置目录", "nginx", "sites-enabled 目录完整路径", 2),
        ("nginx.ssl_dir", "", "SSL 证书目录", "nginx", "证书存放目录完整路径", 3),
        ("nginx.default_root", "", "站点默认根目录", "nginx", "新站点默认根目录完整路径", 4),
        ("nginx.log_access", "", "Access 日志路径", "nginx", "access.log 完整路径", 5),
        ("nginx.log_error", "", "Error 日志路径", "nginx", "error.log 完整路径", 6),
    ];
    for (k, v, n, gc, r, s) in nginx_params {
        if param_dao::find_param_by_key(pool, k).await?.is_none() {
            let _ = param_dao::insert_param_returning_id(pool, k, Some(v), n, gc, Some(r), s).await?;
        }
    }
    Ok(())
}

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