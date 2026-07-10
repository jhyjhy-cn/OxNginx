use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 角色实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub remark: Option<String>,
    pub status: i32, // 1=启用 0=禁用
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}