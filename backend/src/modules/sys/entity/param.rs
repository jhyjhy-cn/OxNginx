use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 系统参数
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Param {
    pub id: i64,
    pub key: String,
    pub value: Option<String>,
    pub name: String,
    pub group_code: String,
    pub remark: Option<String>,
    pub sort: i32,
    pub version: i32,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}