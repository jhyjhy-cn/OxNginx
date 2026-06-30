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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::database::Database;

/// 应用共享状态
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // 初始化数据库
    let db = Database::new(&config.database.path).await?;
    tracing::info!("数据库初始化完成");

    // 创建应用状态
    let state = AppState {
        db,
        config: config.clone(),
    };

    // 公开路由（无需认证）
    let public_routes = Router::new()
        .route("/api/login", post(api::auth_api::login))
        .route("/api/setup", post(api::auth_api::setup));

    // 需要认证的路由
    let protected_routes = Router::new()
        // Dashboard
        .route("/api/dashboard", get(api::dashboard_api::get_dashboard))
        // 站点管理
        .route("/api/sites", get(api::site_api::list_sites))
        .route("/api/sites", post(api::site_api::create_site))
        .route("/api/sites/{id}", get(api::site_api::get_site))
        .route("/api/sites/{id}", put(api::site_api::update_site))
        .route("/api/sites/{id}", delete(api::site_api::delete_site))
        // SSL证书
        .route("/api/certificates", get(api::auth_api::list_certificates))
        .route("/api/certificate/apply", post(api::auth_api::apply_certificate))
        .route("/api/certificate/renew", post(api::auth_api::renew_certificate))
        // Nginx操作
        .route("/api/nginx/test", post(api::nginx_api::test_config))
        .route("/api/nginx/reload", post(api::nginx_api::reload))
        // 日志
        .route("/api/log/access", get(api::log_api::access_log))
        .route("/api/log/error", get(api::log_api::error_log))
        // 备份
        .route("/api/backups/{site_id}", get(api::backup_api::list_backups))
        .route("/api/backups/{site_id}", post(api::backup_api::create_backup))
        .route("/api/backups/restore/{id}", post(api::backup_api::restore_backup))
        .layer(from_fn_with_state(state.clone(), middleware::auth_middleware));

    // 构建路由
    let app = public_routes
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 启动服务
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("OxNginx 启动于 http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
