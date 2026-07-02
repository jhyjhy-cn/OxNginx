use serde::{Deserialize, Serialize};

/// 统一API响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".into(),
            data: Some(data),
        }
    }

    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            code: -1,
            message: message.into(),
            data: None,
        }
    }
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

/// 修改密码请求
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// 创建站点请求
#[derive(Debug, Deserialize)]
pub struct CreateSiteRequest {
    pub name: String,
    pub server_name: String,
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default)]
    pub ssl: bool,
    pub certificate_path: Option<String>,
    pub key_path: Option<String>,
    pub proxy_pass: Option<String>,
    pub root_path: Option<String>,
}

fn default_listen() -> String {
    "80".into()
}

/// 删除站点请求
#[derive(Debug, Deserialize)]
pub struct DeleteSiteRequest {
    #[serde(default = "default_true")]
    pub delete_record: bool,
    #[serde(default)]
    pub delete_files: bool,
}

fn default_true() -> bool {
    true
}

/// 更新站点请求
#[derive(Debug, Deserialize)]
pub struct UpdateSiteRequest {
    pub name: Option<String>,
    pub server_name: Option<String>,
    pub listen: Option<String>,
    pub ssl: Option<bool>,
    pub certificate_path: Option<String>,
    pub key_path: Option<String>,
    pub proxy_pass: Option<String>,
    pub root_path: Option<String>,
    pub status: Option<String>,
}

/// Dashboard数据
#[derive(Debug, Serialize)]
pub struct DashboardData {
    pub nginx_version: String,
    pub worker_count: u32,
    pub active_connections: u64,
    pub site_count: i64,
    pub cert_count: i64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub app_memory: u64, // 本程序占用内存 MB
}

/// 日志响应
#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub lines: Vec<String>,
}

/// SSL申请请求
#[derive(Debug, Deserialize)]
pub struct ApplyCertRequest {
    pub domain: String,
}

/// Nginx测试响应
#[derive(Debug, Serialize)]
pub struct NginxTestResult {
    pub success: bool,
    pub message: String,
}

/// 创建上游服务器请求
#[derive(Debug, Deserialize)]
pub struct CreateUpstreamRequest {
    pub name: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default = "default_keepalive")]
    pub keepalive: i32,
    pub servers: Vec<UpstreamServerRequest>,
}

fn default_method() -> String {
    "round_robin".into()
}

fn default_keepalive() -> i32 {
    32
}

/// 上游服务器节点请求
#[derive(Debug, Deserialize)]
pub struct UpstreamServerRequest {
    pub address: String,
    #[serde(default = "default_weight")]
    pub weight: i32,
    #[serde(default = "default_max_fails")]
    pub max_fails: i32,
    #[serde(default = "default_fail_timeout")]
    pub fail_timeout: String,
    #[serde(default)]
    pub backup: bool,
}

fn default_weight() -> i32 { 1 }
fn default_max_fails() -> i32 { 3 }
fn default_fail_timeout() -> String { "30s".into() }

/// 更新上游服务器请求
#[derive(Debug, Deserialize)]
pub struct UpdateUpstreamRequest {
    pub name: Option<String>,
    pub method: Option<String>,
    pub keepalive: Option<i32>,
    pub status: Option<String>,
    pub servers: Option<Vec<UpstreamServerRequest>>,
}

/// 创建访问控制规则请求
#[derive(Debug, Deserialize)]
pub struct CreateAccessRuleRequest {
    pub site_id: Option<i64>,
    pub rule_type: String,
    pub value: String,
    pub description: Option<String>,
}

/// 更新访问控制规则请求
#[derive(Debug, Deserialize)]
pub struct UpdateAccessRuleRequest {
    pub site_id: Option<i64>,
    pub rule_type: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

/// 创建配置模板请求
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub config: String,
    pub variables: Option<String>,
}

/// 更新配置模板请求
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<String>,
    pub variables: Option<String>,
}

