use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::common::audit::event::AuditEvent;
use crate::modules::log::dao::log_dao;
use crate::modules::log::entity::log::{LoginLog, OperationLog};

#[derive(Debug)]
pub struct OperationLogQuery {
    pub trace_id: Option<String>,
    pub module: Option<String>,
    pub page: i64,
    pub page_size: i64,
    pub username: Option<String>,
    pub status: Option<i32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// 多行 VALUES 一次插入
pub async fn log_operations_batch_multirow(
    pool: &SqlitePool,
    events: &[AuditEvent],
) -> Result<()> {
    Ok(log_dao::insert_operation_logs_multirow(pool, events).await?)
}

/// 单条插入（batch 失败时的降级路径）
pub async fn log_operation_single(pool: &SqlitePool, ev: &AuditEvent) -> Result<()> {
    Ok(log_dao::insert_operation_log_single(pool, ev).await?)
}

pub async fn list_operation_logs(
    pool: &SqlitePool,
    q: &OperationLogQuery,
) -> Result<(Vec<OperationLog>, i64)> {
    let offset = (q.page - 1).max(0) * q.page_size;
    Ok(log_dao::list_operation_logs(
        pool,
        q.username.as_deref(),
        q.status,
        q.start_time.as_deref(),
        q.end_time.as_deref(),
        q.trace_id.as_deref(),
        q.module.as_deref(),
        q.page_size,
        offset,
    )
    .await?)
}

pub async fn export_operation_logs_csv(
    pool: &SqlitePool,
    q: &OperationLogQuery,
) -> Result<String> {
    let (list, _) = list_operation_logs(pool, q).await?;
    let mut csv =
        String::from("\u{FEFF}操作模块,操作类型,请求方式,操作人员,操作地址,操作状态,操作日期,消耗时间(ms),TraceID\n");
    for row in &list {
        let duration = row.duration_ms.or(row.cost_ms).unwrap_or(0);
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

#[allow(clippy::too_many_arguments)]
use crate::modules::common::enums::{LoginLogType, LogStatus};

pub async fn log_login(
    pool: &SqlitePool,
    username: &str,
    ip: Option<&str>,
    os: Option<&str>,
    browser: Option<&str>,
    user_agent: Option<&str>,
    log_type: LoginLogType,
    status: LogStatus,
) -> Result<()> {
    Ok(log_dao::insert_login_log(pool, username, ip, os, browser, user_agent, log_type, status).await?)
}

#[derive(Debug)]
pub struct LoginLogQuery {
    pub page: i64,
    pub page_size: i64,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub status: Option<i32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

pub async fn list_login_logs(
    pool: &SqlitePool,
    q: &LoginLogQuery,
) -> Result<(Vec<LoginLog>, i64)> {
    let offset = (q.page - 1).max(0) * q.page_size;
    Ok(log_dao::list_login_logs(
        pool,
        q.username.as_deref(),
        q.ip.as_deref(),
        q.status,
        q.start_time.as_deref(),
        q.end_time.as_deref(),
        q.page_size,
        offset,
    )
    .await?)
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