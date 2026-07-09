use sqlx::SqlitePool;

use crate::modules::common::audit::event::AuditEvent;
use crate::modules::log::entity::log::{LoginLog, OperationLog};

/// 多行 VALUES 一次插入。50 条事件对应一条 SQL。
pub async fn insert_operation_logs_multirow(
    pool: &SqlitePool,
    events: &[AuditEvent],
) -> sqlx::Result<()> {
    if events.is_empty() {
        return Ok(());
    }
    let n = events.len();
    let placeholders = (0..n)
        .map(|_| "(?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "INSERT INTO sys_operation_logs (trace_id, username, module, action, method, uri, ip, status, duration_ms, request_body, response_body, error_msg, created_at) VALUES {}",
        placeholders
    );
    let mut q = sqlx::query(&sql);
    for ev in events {
        q = q
            .bind(&ev.trace_id)
            .bind(&ev.username)
            .bind(&ev.module)
            .bind(&ev.action)
            .bind(&ev.method)
            .bind(&ev.uri)
            .bind(ev.ip.as_deref())
            .bind(&ev.status)
            .bind(ev.duration_ms)
            .bind(ev.request_body.as_deref())
            .bind(ev.response_body.as_deref())
            .bind(ev.error_msg.as_deref())
            .bind(ev.created_at);
    }
    q.execute(pool).await?;
    Ok(())
}

/// 单条插入（batch 失败时的降级路径）。
pub async fn insert_operation_log_single(
    pool: &SqlitePool,
    ev: &AuditEvent,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO sys_operation_logs (trace_id, username, module, action, method, uri, ip, status, duration_ms, request_body, response_body, error_msg, created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)"
    )
    .bind(&ev.trace_id)
    .bind(&ev.username)
    .bind(&ev.module)
    .bind(&ev.action)
    .bind(&ev.method)
    .bind(&ev.uri)
    .bind(ev.ip.as_deref())
    .bind(&ev.status)
    .bind(ev.duration_ms)
    .bind(ev.request_body.as_deref())
    .bind(ev.response_body.as_deref())
    .bind(ev.error_msg.as_deref())
    .bind(ev.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn list_operation_logs(
    pool: &SqlitePool,
    username: Option<&str>,
    status: Option<&str>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    trace_id: Option<&str>,
    module: Option<&str>,
    page_size: i64,
    offset: i64,
) -> sqlx::Result<(Vec<OperationLog>, i64)> {
    let mut wheres = Vec::new();
    if username.is_some() {
        wheres.push("username LIKE ?");
    }
    if status.is_some() {
        wheres.push("status = ?");
    }
    if start_time.is_some() {
        wheres.push("created_at >= ?");
    }
    if end_time.is_some() {
        wheres.push("created_at <= ?");
    }
    if trace_id.is_some() {
        wheres.push("trace_id = ?");
    }
    if module.is_some() {
        wheres.push("module = ?");
    }
    let where_sql = if wheres.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", wheres.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) FROM sys_operation_logs{}", where_sql);
    let list_sql = format!(
        "SELECT * FROM sys_operation_logs{} ORDER BY id DESC LIMIT ? OFFSET ?",
        where_sql
    );

    let mut count_q = sqlx::query_scalar(&count_sql);
    let mut list_q = sqlx::query_as::<_, OperationLog>(&list_sql);
    if let Some(v) = username {
        let pat = format!("%{}%", v);
        count_q = count_q.bind(pat.clone());
        list_q = list_q.bind(pat);
    }
    if let Some(v) = status {
        count_q = count_q.bind(v);
        list_q = list_q.bind(v);
    }
    if let Some(v) = start_time {
        count_q = count_q.bind(v);
        list_q = list_q.bind(v);
    }
    if let Some(v) = end_time {
        count_q = count_q.bind(v);
        list_q = list_q.bind(v);
    }
    if let Some(v) = trace_id {
        count_q = count_q.bind(v);
        list_q = list_q.bind(v);
    }
    if let Some(v) = module {
        count_q = count_q.bind(v);
        list_q = list_q.bind(v);
    }

    let total: i64 = count_q.fetch_one(pool).await?;
    let list = list_q.bind(page_size).bind(offset).fetch_all(pool).await?;
    Ok((list, total))
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_login_log(
    pool: &SqlitePool,
    username: &str,
    ip: Option<&str>,
    os: Option<&str>,
    browser: Option<&str>,
    user_agent: Option<&str>,
    log_type: &str,
    status: &str,
) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO sys_login_logs (username, ip, os, browser, user_agent, type, status) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(username).bind(ip).bind(os).bind(browser).bind(user_agent).bind(log_type).bind(status)
        .execute(pool).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn list_login_logs(
    pool: &SqlitePool,
    username: Option<&str>,
    ip: Option<&str>,
    status: Option<&str>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    page_size: i64,
    offset: i64,
) -> sqlx::Result<(Vec<LoginLog>, i64)> {
    let mut wheres = Vec::new();
    if username.is_some() {
        wheres.push("username LIKE ?");
    }
    if ip.is_some() {
        wheres.push("ip LIKE ?");
    }
    if status.is_some() {
        wheres.push("status = ?");
    }
    if start_time.is_some() {
        wheres.push("created_at >= ?");
    }
    if end_time.is_some() {
        wheres.push("created_at <= ?");
    }
    let where_sql = if wheres.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", wheres.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) FROM sys_login_logs{}", where_sql);
    let list_sql = format!(
        "SELECT id, username, ip, os, browser, type, status, created_at FROM sys_login_logs{} ORDER BY id DESC LIMIT ? OFFSET ?",
        where_sql
    );

    let mut count_q = sqlx::query_scalar(&count_sql);
    if let Some(v) = username {
        count_q = count_q.bind(format!("%{}%", v));
    }
    if let Some(v) = ip {
        count_q = count_q.bind(format!("%{}%", v));
    }
    if let Some(v) = status {
        count_q = count_q.bind(v);
    }
    if let Some(v) = start_time {
        count_q = count_q.bind(v);
    }
    if let Some(v) = end_time {
        count_q = count_q.bind(v);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let mut list_q = sqlx::query_as::<_, LoginLog>(&list_sql);
    if let Some(v) = username {
        list_q = list_q.bind(format!("%{}%", v));
    }
    if let Some(v) = ip {
        list_q = list_q.bind(format!("%{}%", v));
    }
    if let Some(v) = status {
        list_q = list_q.bind(v);
    }
    if let Some(v) = start_time {
        list_q = list_q.bind(v);
    }
    if let Some(v) = end_time {
        list_q = list_q.bind(v);
    }
    let list = list_q.bind(page_size).bind(offset).fetch_all(pool).await?;
    Ok((list, total))
}