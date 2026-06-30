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
