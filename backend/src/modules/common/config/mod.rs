use serde::Deserialize;

/// 应用配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub acme: AcmeConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub log: LogConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    /// 是否输出 SQL 语句日志，默认关闭
    #[serde(default = "default_log_sql")]
    pub log_sql: bool,
}

fn default_log_sql() -> bool {
    false
}

#[derive(Debug, Deserialize, Clone)]
pub struct AcmeConfig {
    pub bin: String,
    /// acme.sh 证书输出目录（如 /root/.acme.sh）
    #[serde(default = "default_acme_home")]
    pub home: String,
}

fn default_acme_home() -> String {
    "/root/.acme.sh".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    /// Token 过期时间（小时），默认 24
    #[serde(default = "default_token_expires_hours")]
    pub token_expires_hours: u64,
}

fn default_token_expires_hours() -> u64 {
    24
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    /// 日志级别：trace / debug / info / warn / error
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 单个日志文件最大大小（MB），超过后自动轮转
    #[serde(default = "default_log_max_size")]
    pub max_size_mb: u64,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: default_log_level(),
            max_size_mb: default_log_max_size(),
        }
    }
}

fn default_log_level() -> String {
    "debug".to_string()
}

fn default_log_max_size() -> u64 {
    10
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

/// 获取运行目录：debug 构建用项目目录（CARGO_MANIFEST_DIR），release 用 exe 所在目录
pub fn get_run_dir() -> std::path::PathBuf {
    if cfg!(debug_assertions) {
        std::env::var("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::current_exe()
                    .map(|p| p.parent().unwrap_or(&p).to_path_buf())
                    .unwrap_or_default()
            })
    } else {
        std::env::current_exe()
            .map(|p| p.parent().unwrap_or(&p).to_path_buf())
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
    }
}
