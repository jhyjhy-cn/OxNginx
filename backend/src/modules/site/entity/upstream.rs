use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 上游服务器实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Upstream {
    pub id: i64,
    pub name: String,
    pub method: String,
    pub keepalive: i32,
    pub status: i32, // 1=启用 0=禁用
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}

/// 上游服务器节点实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UpstreamServer {
    pub id: i64,
    pub upstream_id: i64,
    pub address: String,
    pub weight: i32,
    pub max_fails: i32,
    pub fail_timeout: String,
    pub backup: i32,
    pub status: i32, // 1=启用 0=禁用
}