use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 反向代理实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReverseProxy {
    pub id: i64,
    pub site_id: i64,
    pub name: String,
    pub proxy_dir: String,
    pub target_url: String,
    pub cache: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}