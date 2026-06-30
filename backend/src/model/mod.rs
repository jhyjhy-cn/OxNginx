use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 用户模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// 站点模型
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
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// 证书模型
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

/// 备份模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Backup {
    pub id: i64,
    pub site_id: Option<i64>,
    pub version: i32,
    pub config: String,
    pub created_at: Option<NaiveDateTime>,
}
