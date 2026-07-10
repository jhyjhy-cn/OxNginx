use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 站点实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub server_name: String,
    pub listen: String,
    pub ssl: i32,
    pub certificate_path: Option<String>,
    pub key_path: Option<String>,
    pub proxy_pass: Option<String>,
    pub root_path: Option<String>,
    pub config: Option<String>,
    pub remark: Option<String>,
    pub expire_time: Option<String>,
    pub rewrite_rules: Option<String>,
    pub redirect_rules: Option<String>,
    pub hotlink_config: Option<String>,
    pub log_access_path: Option<String>,
    pub log_error_path: Option<String>,
    pub status: i32, // 1=启用 0=禁用
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}