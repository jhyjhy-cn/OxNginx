use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 数据库连接实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Database {
    pub id: i64,
    pub r#type: String, // 'redis' | 'sqlite'
    pub name: String,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub db_name: Option<String>,
    pub db_path: Option<String>,
    pub enabled: i32,
    pub sort: i32,
    pub remark: Option<String>,
    pub version: i32,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}

/// 数据库探测结果
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DbTestResult {
    pub running: bool,
    pub not_installed: bool,
    pub version: Option<String>,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

/// 创建请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateDatabaseRequest {
    pub r#type: String,
    pub name: String,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub db_name: Option<String>,
    pub db_path: Option<String>,
    pub enabled: Option<bool>,
    pub sort: Option<i32>,
    pub remark: Option<String>,
}

/// 更新请求(密码空字符串=不修改)
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateDatabaseRequest {
    pub r#type: Option<String>,
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub db_name: Option<String>,
    pub db_path: Option<String>,
    pub enabled: Option<bool>,
    pub sort: Option<i32>,
    pub remark: Option<String>,
}
