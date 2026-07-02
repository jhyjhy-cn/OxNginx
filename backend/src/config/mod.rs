use serde::Deserialize;

/// 应用配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub nginx: NginxConfig,
    pub acme: AcmeConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NginxConfig {
    pub bin: String,
    pub config: String,
    pub sites_enabled: String,
    /// SSL 证书存放目录
    #[serde(default = "default_ssl_dir")]
    pub ssl_dir: String,
    /// 新站点默认根目录
    #[serde(default = "default_root")]
    pub default_root: String,
    /// Nginx access 日志路径
    #[serde(default = "default_log_access")]
    pub log_access: String,
    /// Nginx error 日志路径
    #[serde(default = "default_log_error")]
    pub log_error: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AcmeConfig {
    pub bin: String,
    /// acme.sh 证书输出目录（如 /root/.acme.sh）
    #[serde(default = "default_acme_home")]
    pub home: String,
}

fn default_ssl_dir() -> String {
    #[cfg(target_os = "linux")]
    { "/opt/oxnginx/ssl".to_string() }
    #[cfg(target_os = "windows")]
    { "ssl".to_string() }
}

fn default_root() -> String {
    #[cfg(target_os = "linux")]
    { "/opt/oxnginx/wwwroot".to_string() }
    #[cfg(target_os = "windows")]
    { "wwwroot".to_string() }
}

fn default_log_access() -> String {
    #[cfg(target_os = "linux")]
    { "/opt/oxnginx/wwwlogs/access.log".to_string() }
    #[cfg(target_os = "windows")]
    { "logs/access.log".to_string() }
}

fn default_log_error() -> String {
    #[cfg(target_os = "linux")]
    { "/opt/oxnginx/wwwlogs/error.log".to_string() }
    #[cfg(target_os = "windows")]
    { "logs/error.log".to_string() }
}

fn default_acme_home() -> String {
    "/root/.acme.sh".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expires_hours: u64,
}

impl AppConfig {
    /// 从config.toml加载配置
    pub fn load() -> anyhow::Result<Self> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".into());
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("读取配置文件失败: {}", e))?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
