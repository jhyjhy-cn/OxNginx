use chrono::NaiveDateTime;
use serde::Serialize;

use crate::modules::common::util::datetime::option_naive_datetime;

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
    pub status: i32, // 1=启用 0=禁用
    pub cost_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_msg: Option<String>,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
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
    pub log_type: i32,
    pub status: i32, // 1=启用 0=禁用
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
}