mod api;
mod auth;
mod backup;
mod config;
mod database;
mod dto;
mod middleware;
mod model;
mod nginx;
mod service;
mod ssl;
mod util;

use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::database::Database;
use std::sync::{Arc, Mutex};
use sysinfo::{System, Pid};

/// ponytail: minimal system info for memory efficiency
fn create_system() -> System {
    System::new()
}

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
}

fn main() -> anyhow::Result<()> {
    // 单线程 runtime，大幅降低内存占用（<10MB vs multi-thread 28MB+）
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ox_nginx=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = AppConfig::load()?;
    tracing::info!("配置加载完成");

    rt.block_on(async {
        // 初始化数据库
        let db = Database::new(&config.database.path).await?;
        tracing::info!("数据库初始化完成");

        // 创建应用状态
        let state = AppState {
            db,
            config: Arc::new(Mutex::new(config.clone())),
            sys: Arc::new(Mutex::new(create_system())),
            pid: Pid::from_u32(std::process::id()),
        };

        // 公开路由（无需认证）
        let public_routes = Router::new()
            .route("/api/login", post(api::auth_api::login))
            .route("/api/setup", post(api::auth_api::setup));

        // 需要认证的路由
        let protected_routes = Router::new()
            .route("/api/dashboard", get(api::dashboard_api::get_dashboard))
            .route("/api/sites", get(api::site_api::list_sites))
            .route("/api/sites", post(api::site_api::create_site))
            .route("/api/sites/:id", get(api::site_api::get_site))
            .route("/api/sites/:id", put(api::site_api::update_site))
            .route("/api/sites/:id", delete(api::site_api::delete_site))
            .route("/api/sites/batch/enable", post(api::site_api::batch_enable))
            .route("/api/sites/batch/disable", post(api::site_api::batch_disable))
            .route("/api/sites/batch/delete", post(api::site_api::batch_delete))
            .route("/api/certificates", get(api::auth_api::list_certificates))
            .route("/api/certificate/apply", post(api::auth_api::apply_certificate))
            .route("/api/certificate/renew", post(api::auth_api::renew_certificate))
            .route("/api/nginx/test", post(api::nginx_api::test_config))
            .route("/api/nginx/reload", post(api::nginx_api::reload))
            .route("/api/nginx/status", get(api::nginx_api::status))
            .route("/api/nginx/start", post(api::nginx_api::start))
            .route("/api/nginx/stop", post(api::nginx_api::stop))
            .route("/api/nginx/restart", post(api::nginx_api::restart))
            .route("/api/nginx/install", post(api::nginx_api::install))
            .route("/api/log/access", get(api::log_api::access_log))
            .route("/api/log/error", get(api::log_api::error_log))
            .route("/api/backups/:id", get(api::backup_api::list_backups))
            .route("/api/backups/:id", post(api::backup_api::create_backup))
            .route("/api/backups/:id", delete(api::backup_api::delete_backup))
            .route("/api/backups/restore/:id", post(api::backup_api::restore_backup))
            .route("/api/backups/diff", post(api::backup_api::diff_backups))
            .route("/api/config/main", get(api::config_api::get_main_config))
            .route("/api/config/main", put(api::config_api::save_main_config))
            .route("/api/config/files", get(api::config_api::list_config_files))
            .route("/api/config/file/:name", get(api::config_api::get_site_config))
            .route("/api/config/file/:name", put(api::config_api::save_site_config))
            .route("/api/config/file/:name/toggle", post(api::config_api::toggle_site_config))
            .route("/api/config/file/:name", delete(api::config_api::delete_site_config))
            .route("/api/upstreams", get(api::upstream_api::list_upstreams))
            .route("/api/upstreams", post(api::upstream_api::create_upstream))
            .route("/api/upstreams/:id", get(api::upstream_api::get_upstream))
            .route("/api/upstreams/:id", put(api::upstream_api::update_upstream))
            .route("/api/upstreams/:id", delete(api::upstream_api::delete_upstream))
            .route("/api/access-rules", get(api::access_api::list_rules))
            .route("/api/access-rules", post(api::access_api::create_rule))
            .route("/api/access-rules/:id", get(api::access_api::get_rule))
            .route("/api/access-rules/:id", put(api::access_api::update_rule))
            .route("/api/access-rules/:id", delete(api::access_api::delete_rule))
            .route("/api/templates", get(api::template_api::list_templates))
            .route("/api/templates", post(api::template_api::create_template))
            .route("/api/templates/:id", get(api::template_api::get_template))
            .route("/api/templates/:id", put(api::template_api::update_template))
            .route("/api/templates/:id", delete(api::template_api::delete_template))
            .route("/api/templates/:id/preview", post(api::template_api::preview_template))
            .route("/api/settings", get(api::settings_api::get_settings))
            .route("/api/settings", put(api::settings_api::update_settings))
            .layer(from_fn_with_state(state.clone(), middleware::auth_middleware));

        // 静态文件服务（前端）
        // SPA 路由：所有未匹配的路径都返回 index.html，由 Vue Router 处理前端路由
        let static_service = ServeDir::new("static")
            .not_found_service(ServeFile::new("static/index.html"));

        // 构建路由
        let app = public_routes
            .merge(protected_routes)
            .layer(CorsLayer::permissive())
            .fallback_service(static_service)
            .with_state(state);

        // 启动服务
        let addr = format!("{}:{}", config.server.host, config.server.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::info!("OxNginx 启动于 http://{}", addr);

        axum::serve(listener, app).await?;

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
