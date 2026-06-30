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
}

#[derive(Debug, Deserialize, Clone)]
pub struct AcmeConfig {
    pub bin: String,
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
