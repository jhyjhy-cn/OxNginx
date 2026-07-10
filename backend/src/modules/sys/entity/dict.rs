use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 字典
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dict {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: i32, // 1=启用 0=禁用
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}

/// 字典项
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DictItem {
    pub id: i64,
    pub dict_id: i64,
    pub label: String,
    pub value: String,
    pub sort: i32,
    pub status: i32, // 1=启用 0=禁用
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}