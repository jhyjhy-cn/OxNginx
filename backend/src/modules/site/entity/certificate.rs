use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 证书实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Certificate {
    pub id: i64,
    pub domain: String,
    pub issuer: Option<String>,
    pub expire_time: Option<NaiveDateTime>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub auto_renew: i32,
    pub created_at: Option<NaiveDateTime>,
}