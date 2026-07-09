use crate::modules::common::config::AppConfig;
use crate::modules::common::database::Database;
use std::sync::{Arc, Mutex};
use sysinfo::{System, Pid};
use tokio::sync::broadcast;

pub struct AppState {
    pub db: Database,
    pub config: Arc<Mutex<AppConfig>>,
    pub sys: Arc<Mutex<System>>,
    pub pid: Pid,
    pub dashboard_tx: broadcast::Sender<String>,
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
            rsa_private_key: Arc::clone(&self.rsa_private_key),
            rsa_public_key_b64: self.rsa_public_key_b64.clone(),
        }
    }
}

impl AppState {
    pub fn get_config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn update_config(&self, new_config: AppConfig) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
    }

    pub fn new(db: Database, config: AppConfig, rsa_private_key: rsa::RsaPrivateKey, rsa_public_key_b64: String) -> Self {
        let (dashboard_tx, _) = broadcast::channel(16);
        AppState {
            db,
            config: Arc::new(Mutex::new(config)),
            sys: Arc::new(Mutex::new(System::new())),
            pid: Pid::from_u32(std::process::id()),
            dashboard_tx,
            rsa_private_key: Arc::new(rsa_private_key),
            rsa_public_key_b64,
        }
    }
}
