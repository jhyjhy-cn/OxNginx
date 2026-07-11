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

pub async fn list_operation_logs_for_export(
    pool: &SqlitePool,
    q: &OperationLogQuery,
) -> Result<Vec<OperationLog>> {
    Ok(log_dao::list_operation_logs_for_export(
        pool,
        q.username.as_deref(),
        q.status,
        q.start_time.as_deref(),
        q.end_time.as_deref(),
        q.trace_id.as_deref(),
        q.module.as_deref(),
    )
    .await?)
}

pub async fn export_operation_logs_xlsx(
    pool: &SqlitePool,
    q: &OperationLogQuery,
) -> Result<Vec<u8>> {
    use crate::modules::common::util::excel::{build_xlsx, Sheet};
    let list = list_operation_logs_for_export(pool, q).await?;
    let headers = vec![
        "操作模块".to_string(),
        "操作类型".to_string(),
        "请求方式".to_string(),
        "操作人员".to_string(),
        "操作地址".to_string(),
        "操作状态".to_string(),
        "操作日期".to_string(),
        "消耗时间(ms)".to_string(),
        "TraceID".to_string(),
    ];
    let rows: Vec<Vec<String>> = list
        .iter()
        .map(|row| {
            let duration = row.duration_ms.or(row.cost_ms).unwrap_or(0);
            vec![
                row.module.as_deref().unwrap_or("").to_string(),
                row.action.clone(),
                row.method.as_deref().unwrap_or("").to_string(),
                row.username.clone(),
                row.uri.as_deref().unwrap_or("").to_string(),
                row.status.to_string(),
                row.created_at
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
                duration.to_string(),
                row.trace_id.as_deref().unwrap_or("").to_string(),
            ]
        })
        .collect();
    let sheet = Sheet { headers, rows };
    build_xlsx("操作日志", &sheet)
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

pub async fn list_login_logs_for_export(pool: &SqlitePool, q: &LoginLogQuery) -> Result<Vec<LoginLog>> {
    Ok(log_dao::list_login_logs_for_export(
        pool,
        q.username.as_deref(),
        q.ip.as_deref(),
        q.status,
        q.start_time.as_deref(),
        q.end_time.as_deref(),
    )
    .await?)
}

pub async fn export_login_logs_xlsx(pool: &SqlitePool, q: &LoginLogQuery) -> Result<Vec<u8>> {
    use crate::modules::common::util::excel::{build_xlsx, Sheet};
    let list = list_login_logs_for_export(pool, q).await?;
    let headers = vec![
        "用户名".to_string(),
        "IP".to_string(),
        "操作系统".to_string(),
        "浏览器".to_string(),
        "类型".to_string(),
        "状态".to_string(),
        "时间".to_string(),
    ];
    let rows: Vec<Vec<String>> = list
        .iter()
        .map(|row| {
            vec![
                row.username.clone(),
                row.ip.as_deref().unwrap_or("").to_string(),
                row.os.as_deref().unwrap_or("").to_string(),
                row.browser.as_deref().unwrap_or("").to_string(),
                row.log_type.to_string(),
                row.status.to_string(),
                row.created_at
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
            ]
        })
        .collect();
    let sheet = Sheet { headers, rows };
    build_xlsx("登录日志", &sheet)
}