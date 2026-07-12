use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 备份实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Backup {
    pub id: i64,
    pub site_id: Option<i64>,
    pub version: i32, // 备份版本号（非乐观锁）
    pub config: String,
    pub remark: Option<String>,
    pub sort: i32,
    pub dept_id: Option<i64>,
    pub is_deleted: i32,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}