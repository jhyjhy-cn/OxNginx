use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 上游服务器实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Upstream {
    pub id: i64,
    pub name: String,
    pub method: String,
    pub keepalive: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
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
    pub status: String,
}