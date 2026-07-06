pub mod tray;
pub mod window;

use std::sync::{Arc, Mutex};
use crate::config::AppConfig;
use crate::database::Database;
use crate::AppState;

/// 启动GUI（在后台线程启动服务，主线程运行GUI窗口和托盘）
pub fn run_gui(config: AppConfig) -> anyhow::Result<()> {
    // 初始化服务状态
    let service_running = Arc::new(Mutex::new(false));

    // 在后台线程启动异步服务
    let config_clone = config.clone();
    let running_clone = service_running.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async {
            let db = match Database::new(&config_clone.database.path).await {
                Ok(db) => {
                    tracing::info!("数据库初始化完成");
                    db
                }
                Err(e) => {
                    tracing::error!("数据库初始化失败: {}", e);
                    return;
                }
            };

            let app_state = AppState::new(db, config_clone.clone());
            let app = crate::app::router::build(app_state);

            *running_clone.lock().unwrap() = true;

            let addr = format!("{}:{}", config_clone.server.host, config_clone.server.port);
            let listener = match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => {
                    tracing::info!("OxNginx 启动于 http://{}", addr);
                    l
                }
                Err(e) => {
                    tracing::error!("绑定地址失败: {}", e);
                    *running_clone.lock().unwrap() = false;
                    return;
                }
            };

            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("服务运行错误: {}", e);
                *running_clone.lock().unwrap() = false;
            }
        });
    });

    // 先创建通道（必须在创建托盘之前）
    let (sender, receiver) = std::sync::mpsc::channel();
    // 存储全局发送器供托盘线程使用
    let _ = window::WINDOW_SENDER.set(sender);

    // 创建托盘图标（必须在主线程创建）
    tracing::info!("创建系统托盘（主线程）");
    let _tray_icon = tray::create_tray_icon(&config)?;

    // 主线程运行窗口（阻塞直到窗口关闭）
    tracing::info!("启动主窗口（主线程）");
    window::run_window_with_receiver(config, service_running, receiver);

    Ok(())
}
