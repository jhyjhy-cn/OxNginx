use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 反向代理实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReverseProxy {
    pub id: i64,
    pub site_id: i64,
    pub name: String,
    pub proxy_dir: String,
    pub target_url: String,
    pub cache: i32,
    pub status: i32, // 1=启用 0=禁用
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}