use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 系统文件
/// `url` 字段不在 DB 里，list 时由 service 拼接 base_url 后注入（前端直接用）
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct File {
    pub id: i64,
    pub name: String,
    pub original_name: String,
    pub suffix: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub md5: Option<String>,
    pub path: String,
    pub provider: String,
    pub dept_id: Option<i64>,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    /// url 不是 DB 列；sqlx 用 #[sqlx(default)] 跳过，serde 反序列化也跳过
    #[sqlx(default)]
    #[serde(default, skip_deserializing)]
    pub url: Option<String>,
}