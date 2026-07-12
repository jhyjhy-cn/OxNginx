use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 配置模板实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub remark: Option<String>,
    pub config: String,
    pub variables: Option<String>,
    pub sort: i32,
    pub version: i32,
    pub dept_id: Option<i64>,
    pub is_deleted: i32,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub status: i32, // 1=启用 0=禁用
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}