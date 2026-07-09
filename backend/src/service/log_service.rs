use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::audit::event::AuditEvent;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OperationLog {
    pub id: i64,
    pub trace_id: Option<String>,
    pub username: String,
    pub module: Option<String>,
    pub action: String,
    pub method: Option<String>,
    pub uri: Option<String>,
    pub ip: Option<String>,
    pub status: String,
    pub cost_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_msg: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LoginLog {
    pub id: i64,
    pub username: String,
    pub ip: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub log_type: String,
    pub status: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

/// 多行 VALUES 一次插入。50 条事件对应一条 SQL。
pub async fn log_operations_batch_multirow(
    pool: &SqlitePool,
    events: &[AuditEvent],
) -> Result<()> {
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
pub async fn log_operation_single(pool: &SqlitePool, ev: &AuditEvent) -> Result<()> {
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

pub struct OperationLogQuery {
    pub trace_id: Option<String>,
    pub module: Option<String>,
    pub page: i64,
    pub page_size: i64,
    pub username: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

pub async fn list_operation_logs(
    pool: &SqlitePool,
    q: &OperationLogQuery,
) -> Result<(Vec<OperationLog>, i64)> {
    let offset = (q.page - 1).max(0) * q.page_size;
    let mut wheres = Vec::new();
    if q.username.is_some() {
        wheres.push("username LIKE ?");
    }
    if q.status.is_some() {
        wheres.push("status = ?");
    }
    if q.start_time.is_some() {
        wheres.push("created_at >= ?");
    }
    if q.end_time.is_some() {
        wheres.push("created_at <= ?");
    }
    if q.trace_id.is_some() {
        wheres.push("trace_id = ?");
    }
    if q.module.is_some() {
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
    if let Some(ref v) = q.username {
        count_q = count_q.bind(format!("%{}%", v));
        list_q = list_q.bind(format!("%{}%", v));
    }
    if let Some(ref v) = q.status {
        count_q = count_q.bind(v.clone());
        list_q = list_q.bind(v.clone());
    }
    if let Some(ref v) = q.start_time {
        count_q = count_q.bind(v.clone());
        list_q = list_q.bind(v.clone());
    }
    if let Some(ref v) = q.end_time {
        count_q = count_q.bind(v.clone());
        list_q = list_q.bind(v.clone());
    }
    if let Some(ref v) = q.trace_id {
        count_q = count_q.bind(v.clone());
        list_q = list_q.bind(v.clone());
    }
    if let Some(ref v) = q.module {
        count_q = count_q.bind(v.clone());
        list_q = list_q.bind(v.clone());
    }

    let total: i64 = count_q.fetch_one(pool).await?;
    let list = list_q.bind(q.page_size).bind(offset).fetch_all(pool).await?;
    Ok((list, total))
}

pub async fn export_operation_logs_csv(
    pool: &SqlitePool,
    q: &OperationLogQuery,
) -> Result<String> {
    let (list, _) = list_operation_logs(pool, q).await?;
    let mut csv =
        String::from("\u{FEFF}操作模块,操作类型,请求方式,操作人员,操作地址,操作状态,操作日期,消耗时间(ms),TraceID\n");
    for row in &list {
        let duration = row
            .duration_ms
            .or(row.cost_ms)
            .unwrap_or(0);
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            row.module.as_deref().unwrap_or(""),
            row.action,
            row.method.as_deref().unwrap_or(""),
            row.username,
            row.uri.as_deref().unwrap_or(""),
            row.status,
            row.created_at
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
            duration,
            row.trace_id.as_deref().unwrap_or(""),
        ));
    }
    Ok(csv)
}

pub async fn log_login(
    pool: &SqlitePool,
    username: &str,
    ip: Option<&str>,
    os: Option<&str>,
    browser: Option<&str>,
    user_agent: Option<&str>,
    log_type: &str,
    status: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO sys_login_logs (username, ip, os, browser, user_agent, type, status) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(username).bind(ip).bind(os).bind(browser).bind(user_agent).bind(log_type).bind(status)
        .execute(pool).await?;
    Ok(())
}

pub struct LoginLogQuery {
    pub page: i64,
    pub page_size: i64,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

pub async fn list_login_logs(
    pool: &SqlitePool,
    q: &LoginLogQuery,
) -> Result<(Vec<LoginLog>, i64)> {
    let offset = (q.page - 1).max(0) * q.page_size;
    let mut wheres = Vec::new();
    if q.username.is_some() {
        wheres.push("username LIKE ?");
    }
    if q.ip.is_some() {
        wheres.push("ip LIKE ?");
    }
    if q.status.is_some() {
        wheres.push("status = ?");
    }
    if q.start_time.is_some() {
        wheres.push("created_at >= ?");
    }
    if q.end_time.is_some() {
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
    if let Some(ref v) = q.username {
        count_q = count_q.bind(format!("%{}%", v));
    }
    if let Some(ref v) = q.ip {
        count_q = count_q.bind(format!("%{}%", v));
    }
    if let Some(ref v) = q.status {
        count_q = count_q.bind(v.clone());
    }
    if let Some(ref v) = q.start_time {
        count_q = count_q.bind(v.clone());
    }
    if let Some(ref v) = q.end_time {
        count_q = count_q.bind(v.clone());
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let mut list_q = sqlx::query_as::<_, LoginLog>(&list_sql);
    if let Some(ref v) = q.username {
        list_q = list_q.bind(format!("%{}%", v));
    }
    if let Some(ref v) = q.ip {
        list_q = list_q.bind(format!("%{}%", v));
    }
    if let Some(ref v) = q.status {
        list_q = list_q.bind(v.clone());
    }
    if let Some(ref v) = q.start_time {
        list_q = list_q.bind(v.clone());
    }
    if let Some(ref v) = q.end_time {
        list_q = list_q.bind(v.clone());
    }
    let list = list_q.bind(q.page_size).bind(offset).fetch_all(pool).await?;
    Ok((list, total))
}

pub async fn export_login_logs_csv(pool: &SqlitePool, q: &LoginLogQuery) -> Result<String> {
    let (list, _) = list_login_logs(pool, q).await?;
    let mut csv = String::from("\u{FEFF}用户名,IP,操作系统,浏览器,类型,状态,时间\n");
    for row in &list {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            row.username,
            row.ip.as_deref().unwrap_or(""),
            row.os.as_deref().unwrap_or(""),
            row.browser.as_deref().unwrap_or(""),
            row.log_type,
            row.status,
            row.created_at
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        ));
    }
    Ok(csv)
}
