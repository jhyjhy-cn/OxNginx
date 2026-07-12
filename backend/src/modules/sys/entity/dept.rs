use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 部门实体
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Dept {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort: i32,
    pub status: i32, // 1=启用 0=禁用
    pub version: i32,
    pub created_by: Option<i64>,
    pub remark: Option<String>,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}