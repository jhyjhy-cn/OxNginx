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

/// 上游服务器模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Upstream {
    pub id: i64,
    pub name: String,
    pub method: String,
    pub keepalive: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// 上游服务器节点模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UpstreamServer {
    pub id: i64,
    pub upstream_id: i64,
    pub address: String,
    pub weight: i32,
    pub max_fails: i32,
    pub fail_timeout: String,
    pub backup: i32,
    pub status: String,
}

/// 访问控制规则模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccessRule {
    pub id: i64,
    pub site_id: Option<i64>,
    pub rule_type: String,
    pub value: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
}

/// 配置模板模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub config: String,
    pub variables: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
