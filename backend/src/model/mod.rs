use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 用户模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub dept_id: Option<i64>,
    pub post_id: Option<i64>,
    pub disabled: i32,
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
    pub remark: Option<String>,
    pub expire_time: Option<String>,
    pub rewrite_rules: Option<String>,
    pub redirect_rules: Option<String>,
    pub hotlink_config: Option<String>,
    pub log_access_path: Option<String>,
    pub log_error_path: Option<String>,
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

/// 反向代理模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReverseProxy {
    pub id: i64,
    pub site_id: i64,
    pub name: String,
    pub proxy_dir: String,
    pub target_url: String,
    pub cache: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

// ============== RBAC 模型 ==============

/// 角色
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub remark: Option<String>,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// 部门
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dept {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// 岗位
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub sort: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// 菜单/按钮（type: M=目录 C=菜单 F=按钮）
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Menu {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub title: String,
    pub icon: Option<String>,
    pub path: Option<String>,
    pub component: Option<String>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub menu_type: String,
    pub permission: Option<String>,
    pub sort: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// 国际化翻译条目
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct I18nEntry {
    pub id: i64,
    pub locale: String,
    pub key: String,
    pub value: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
