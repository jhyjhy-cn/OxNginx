use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 证书实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Certificate {
    pub id: i64,
    pub domain: String,
    pub issuer: Option<String>,
    #[serde(with = "option_naive_datetime")]
    pub expire_time: Option<NaiveDateTime>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub auto_renew: i32,
    pub remark: Option<String>,
    pub sort: i32,
    pub version: i32,
    pub dept_id: Option<i64>,
    pub is_deleted: i32,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}