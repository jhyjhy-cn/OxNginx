use crate::config::AppConfig;
use crate::database::Database;
use std::sync::{Arc, Mutex};
use sysinfo::{System, Pid};

/// 应用共享状态
pub struct AppState {
    pub db: Database,
    pub config: Arc<Mutex<AppConfig>>,
    pub sys: Arc<Mutex<System>>,
    pub pid: Pid,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        AppState {
            db: self.db.clone(),
            config: Arc::clone(&self.config),
            sys: Arc::clone(&self.sys),
            pid: self.pid,
        }
    }
}

impl AppState {
    /// 获取配置的克隆副本（不跨 await 点持有锁）
    pub fn get_config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    /// 更新配置
    pub fn update_config(&self, new_config: AppConfig) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
    }

    /// 创建新的 AppState 实例
    pub fn new(db: Database, config: AppConfig) -> Self {
        AppState {
            db,
            config: Arc::new(Mutex::new(config)),
            sys: Arc::new(Mutex::new(System::new())),
            pid: Pid::from_u32(std::process::id()),
        }
    }
}
