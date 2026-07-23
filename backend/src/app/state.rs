use crate::modules::common::config::AppConfig;
use crate::modules::common::database::Database;
use parking_lot::RwLock;
use std::sync::Arc;
use sysinfo::{Pid, System};
use tokio::sync::broadcast;

pub struct AppState {
    pub db: Database,
    // ponytail: RwLock 替换 Mutex，配置读多写少；parking_lot 无 poison
    pub config: Arc<RwLock<AppConfig>>,
    pub sys: Arc<RwLock<System>>,
    pub pid: Pid,
    pub dashboard_tx: broadcast::Sender<String>,
    /// 最近一次 dashboard 快照缓存：后台任务每 10s、nginx 操作后刷新；
    /// 客户端订阅时直接回缓存,避免现场采集(wmic/磁盘查询数秒)导致的推送延迟
    pub dashboard_cache: Arc<RwLock<serde_json::Value>>,
    pub event_tx: broadcast::Sender<crate::modules::websocket::protocol::ServerEvent>,
    pub rsa_private_key: Arc<rsa::RsaPrivateKey>,
    pub rsa_public_key_b64: String,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        AppState {
            db: self.db.clone(),
            config: Arc::clone(&self.config),
            sys: Arc::clone(&self.sys),
            pid: self.pid,
            dashboard_tx: self.dashboard_tx.clone(),
            dashboard_cache: Arc::clone(&self.dashboard_cache),
            event_tx: self.event_tx.clone(),
            rsa_private_key: Arc::clone(&self.rsa_private_key),
            rsa_public_key_b64: self.rsa_public_key_b64.clone(),
        }
    }
}

impl AppState {
    /// ponytail: parking_lot RwLock 读锁不返回 Result，调用方更轻
    pub fn get_config(&self) -> AppConfig {
        self.config.read().clone()
    }

    pub fn update_config(&self, new_config: AppConfig) {
        *self.config.write() = new_config;
    }

    pub fn new(db: Database, config: AppConfig, rsa_private_key: rsa::RsaPrivateKey, rsa_public_key_b64: String) -> Self {
        let (dashboard_tx, _) = broadcast::channel(16);
        let (event_tx, _) = broadcast::channel(64);
        AppState {
            db,
            config: Arc::new(RwLock::new(config)),
            sys: Arc::new(RwLock::new(System::new())),
            pid: Pid::from_u32(std::process::id()),
            dashboard_tx,
            dashboard_cache: Arc::new(RwLock::new(serde_json::Value::Null)),
            event_tx,
            rsa_private_key: Arc::new(rsa_private_key),
            rsa_public_key_b64,
        }
    }
}