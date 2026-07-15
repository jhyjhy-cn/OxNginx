use std::time::{Duration, Instant};

use sqlx::SqlitePool;
use tokio::process::Command;

use crate::modules::common::util::cmd::silent_tokio_command;
use crate::modules::database::dao::database_dao;
use crate::modules::database::entity::database::{
    CreateDatabaseRequest, Database, DbTestResult, UpdateDatabaseRequest,
};
use crate::modules::database::service::sqlite_service;
use crate::AppState;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// 列表(可按 type 过滤)
pub async fn list_databases(
    state: &AppState,
    db_type: Option<&str>,
) -> anyhow::Result<Vec<Database>> {
    let pool = state.db.pool();
    let rows = match db_type {
        Some(t) if !t.is_empty() => database_dao::list_by_type(pool, t).await?,
        _ => database_dao::list_all(pool).await?,
    };
    Ok(rows.into_iter().map(mask_password).collect())
}

/// 详情
pub async fn get_database(state: &AppState, id: i64) -> anyhow::Result<Option<Database>> {
    Ok(database_dao::find_by_id(state.db.pool(), id)
        .await?
        .map(mask_password))
}

/// 创建
pub async fn create_database(
    state: &AppState,
    req: CreateDatabaseRequest,
    created_by: Option<i64>,
) -> anyhow::Result<Database> {
    let pool = state.db.pool();
    if database_dao::find_by_name(pool, &req.name).await?.is_some() {
        anyhow::bail!("名称已存在: {}", req.name);
    }
    // ponytail: sqlite 新建模式(db_name=None)要在默认目录建空 db 文件
    let mut req = req;
    if req.r#type == "sqlite" && req.db_name.as_deref().map_or(true, str::is_empty) {
        let default_dir = sqlite_service::default_sqlite_dir();
        tokio::fs::create_dir_all(&default_dir).await?;
        let path = default_dir.join(format!("{}.db", req.name));
        if !path.exists() {
            // 先建空文件,再让 sqlite connect 触发 schema(自动写 header)
            tokio::fs::File::create(&path).await?;
            let url = format!("sqlite://{}", path.display());
            let p = sqlx::SqlitePool::connect(&url).await?;
            p.close().await;
        }
        // 统一正斜杠存,Windows 上展示/复制更友好
        let path_str = path.to_string_lossy().replace('\\', "/");
        req.db_name = Some(path_str);
    }
    let row = database_dao::insert(pool, &req, created_by).await?;
    Ok(mask_password(row))
}

/// 更新(密码字段空字符串=不修改)
pub async fn update_database(
    state: &AppState,
    id: i64,
    req: UpdateDatabaseRequest,
    updated_by: Option<i64>,
) -> anyhow::Result<Option<Database>> {
    let pool = state.db.pool();
    let existing = match database_dao::find_by_id(pool, id).await? {
        Some(e) => e,
        None => return Ok(None),
    };

    let new_type = req.r#type.unwrap_or(existing.r#type.clone());
    let new_name = req.name.clone().unwrap_or_else(|| existing.name.clone());
    if new_name != existing.name {
        if let Some(_) = database_dao::find_by_name(pool, &new_name).await? {
            anyhow::bail!("名称已存在: {}", new_name);
        }
    }
    let host = req.host.or_else(|| existing.host.clone());
    let port = req.port.or(existing.port);
    let username = req.username.or_else(|| existing.username.clone());
    // 密码:req 传空字符串 → 保留原值(空字符串清空密码需传 sentinel,这里简化为非空才改)
    let password: Option<String> = match req.password {
        Some(p) if !p.is_empty() => Some(p),
        _ => existing.password.clone(),
    };
    let db_name = req.db_name.or_else(|| existing.db_name.clone());
    let db_path = req.db_path.or_else(|| existing.db_path.clone());
    let enabled = req
        .enabled
        .map(|v| if v { 1 } else { 0 })
        .unwrap_or(existing.enabled);
    let sort = req.sort.unwrap_or(existing.sort);
    let remark = req.remark.or_else(|| existing.remark.clone());

    let row = database_dao::update(
        pool,
        id,
        &new_type,
        &new_name,
        host.as_ref(),
        port,
        username.as_ref(),
        password.as_deref(),
        db_name.as_ref(),
        db_path.as_ref(),
        enabled,
        sort,
        remark.as_ref(),
        updated_by,
    )
    .await?;
    Ok(row.map(mask_password))
}

/// 启停
pub async fn toggle_database(state: &AppState, id: i64) -> anyhow::Result<Option<Database>> {
    Ok(database_dao::toggle(state.db.pool(), id)
        .await?
        .map(mask_password))
}

/// 删除:先取记录再删,返回被删的记录(None 表示不存在)
pub async fn delete_database(state: &AppState, id: i64) -> anyhow::Result<Option<Database>> {
    let pool = state.db.pool();
    let existing = match database_dao::find_by_id(pool, id).await? {
        Some(d) => d,
        None => return Ok(None),
    };
    database_dao::delete(pool, id).await?;
    Ok(Some(existing))
}

/// 探测连通性
pub async fn test_connection(state: &AppState, id: i64) -> anyhow::Result<DbTestResult> {
    let db = match database_dao::find_by_id(state.db.pool(), id).await? {
        Some(d) => d,
        None => anyhow::bail!("数据库连接不存在: id={}", id),
    };
    Ok(match db.r#type.as_str() {
        "redis" => test_redis(&db).await,
        "sqlite" => test_sqlite(state.db.pool(), &db).await,
        other => DbTestResult {
            running: false,
            not_installed: true,
            version: None,
            latency_ms: None,
            error: Some(format!("不支持的数据库类型: {}", other)),
        },
    })
}

/// 密码脱敏(返回前统一处理)
fn mask_password(mut db: Database) -> Database {
    if db.password.is_some() {
        db.password = Some("******".to_string());
    }
    db
}

/// Redis 探测:redis-cli ping → PONG
async fn test_redis(db: &Database) -> DbTestResult {
    let start = Instant::now();
    let host = db.host.clone().unwrap_or_else(|| "127.0.0.1".into());
    let port = db.port.unwrap_or(6379);
    let port_str = port.to_string();
    let pwd = db.password.clone().unwrap_or_default();

    let mut args: Vec<String> = vec!["-h".into(), host.clone(), "-p".into(), port_str];
    if !pwd.is_empty() {
        args.push("-a".into());
        args.push(pwd.clone());
    }
    args.push("ping".into());

    let output = tokio::time::timeout(
        PROBE_TIMEOUT,
        silent_tokio_command("redis-cli").args(&args).output(),
    )
    .await;

    let latency = start.elapsed().as_millis() as u64;

    match output {
        Err(_) => DbTestResult {
            running: false,
            not_installed: false,
            version: None,
            latency_ms: Some(latency),
            error: Some("探测超时".into()),
        },
        Ok(Err(_io)) => DbTestResult {
            // ponytail: redis-cli 不存在时区分 timeout vs not_installed;此处 io 错误通常=not found
            running: false,
            not_installed: true,
            version: None,
            latency_ms: Some(latency),
            error: Some("未检测到 redis-cli".into()),
        },
        Ok(Ok(out)) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            if out.status.success() && stdout.contains("PONG") {
                // 拉版本
                let version = fetch_redis_version(&host, port, &pwd).await;
                DbTestResult {
                    running: true,
                    not_installed: false,
                    version,
                    latency_ms: Some(latency),
                    error: None,
                }
            } else {
                DbTestResult {
                    running: false,
                    not_installed: false,
                    version: None,
                    latency_ms: Some(latency),
                    error: Some(format!(
                        "redis 返回非 PONG: {}",
                        stdout.trim()
                    )),
                }
            }
        }
    }
}

/// redis-cli INFO server → 解析 redis_version
async fn fetch_redis_version(host: &str, port: i64, pwd: &str) -> Option<String> {
    let mut args: Vec<String> = vec!["-h".into(), host.into(), "-p".into(), port.to_string()];
    if !pwd.is_empty() {
        args.push("-a".into());
        args.push(pwd.into());
    }
    args.push("INFO".into());
    args.push("server".into());
    let out = tokio::time::timeout(
        PROBE_TIMEOUT,
        silent_tokio_command("redis-cli").args(&args).output(),
    )
    .await
    .ok()?
    .ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        if let Some(v) = line.strip_prefix("redis_version:") {
            return Some(v.trim().to_string());
        }
    }
    // ponytail: 用 `redis-cli --json` 时再升级到 serde_json 解析
    None
}

/// SQLite 探测:db_name 不空走 SqlitePool::connect,空则用项目 pool
async fn test_sqlite(project_pool: &SqlitePool, db: &Database) -> DbTestResult {
    let start = Instant::now();
    let latency = || start.elapsed().as_millis() as u64;

    let target = db.db_name.as_deref().filter(|s| !s.is_empty());

    match target {
        Some(path) => {
            // 临时建一个 pool 探活
            let url = format!("sqlite://{}?mode=ro", path);
            let conn = tokio::time::timeout(
                PROBE_TIMEOUT,
                SqlitePool::connect(&url),
            )
            .await;
            let latency_val = latency();
            match conn {
                Err(_) => DbTestResult {
                    running: false,
                    not_installed: false,
                    version: None,
                    latency_ms: Some(latency_val),
                    error: Some("探测超时".into()),
                },
                Ok(Err(e)) => DbTestResult {
                    running: false,
                    not_installed: false,
                    version: None,
                    latency_ms: Some(latency_val),
                    error: Some(format!("连接失败: {}", e)),
                },
                Ok(Ok(pool)) => {
                    let version = sqlx::query_scalar::<_, String>("SELECT sqlite_version()")
                        .fetch_one(&pool)
                        .await
                        .ok();
                    let _ = pool.close().await;
                    DbTestResult {
                        running: true,
                        not_installed: false,
                        version,
                        latency_ms: Some(latency_val),
                        error: None,
                    }
                }
            }
        }
        None => {
            // 用项目主 pool
            let res = tokio::time::timeout(
                PROBE_TIMEOUT,
                sqlx::query_scalar::<_, String>("SELECT sqlite_version()").fetch_one(project_pool),
            )
            .await;
            let latency_val = latency();
            match res {
                Err(_) => DbTestResult {
                    running: false,
                    not_installed: false,
                    version: None,
                    latency_ms: Some(latency_val),
                    error: Some("探测超时".into()),
                },
                Ok(Err(e)) => DbTestResult {
                    running: false,
                    not_installed: false,
                    version: None,
                    latency_ms: Some(latency_val),
                    error: Some(format!("查询失败: {}", e)),
                },
                Ok(Ok(version)) => DbTestResult {
                    running: true,
                    not_installed: false,
                    version: Some(version),
                    latency_ms: Some(latency_val),
                    error: None,
                },
            }
        }
    }
}

// 抑制未用警告(保留未来扩展)
#[allow(dead_code)]
fn _silence_unused_cmd_import() -> Command {
    Command::new("")
}
