// Windows: 隐藏控制台（仅在release版本生效）
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod api;
mod app;
mod audit;
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
mod startup;
mod util;

// re-export，保持其他模块 `use crate::AppState` 不变
pub use app::state::AppState;

use crate::config::AppConfig;
use crate::database::Database;

const BANNER: &str = r#"
////////////////////////////////////////////////////////////////////
//                          _ooOoo_                               //
//                         o8888888o                              //
//                         88" . "88                              //
//                         (| ^_^ |)                              //
//                         O\  =  /O                              //
//                      ____/`---'\____                           //
//                    .'  \\|     |//  `.                         //
//                   /  \\|||  :  |||//  \                        //
//                  /  _||||| -:- |||||-  \                       //
//                  |   | \\\  -  /// |   |                       //
//                  | \_|  ''\---/''  |   |                       //
//                  \  .-\__  `-`  ___/-. /                       //
//                ___`. .'  /--.--\  `. . ___                     //
//              ."" '<  `.___\_<|>_/___.'  >'"".                  //
//            | | :  `- \`.;`\ _ /`;.`/ - ` : | |                 //
//            \  \ `-.   \_ __\ /__ _/   .-` /  /                 //
//      ========`-.____`-.___\_____/___.-`____.-'========         //
//                           `=---='                              //
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^        //
//             佛祖保佑       永不宕机      永无BUG                 //
////////////////////////////////////////////////////////////////////
"#;

fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let show_console = args.contains(&"--console".to_string()) || args.contains(&"-c".to_string());

    // 如果指定了 --console，附加到父进程控制台（Windows）
    #[cfg(target_os = "windows")]
    if show_console {
        unsafe {
            use winapi::um::consoleapi::AllocConsole;
            AllocConsole();
        }
    }

    println!("{}", BANNER);

    // 首次运行自动初始化（cargo-packager 安装后）
    let exe_dir = std::env::current_exe()?
        .parent().map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    startup::setup::first_run_setup(&exe_dir)?;

    // 先加载配置（首次运行若无配置则自动生成默认配置）
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| exe_dir.join("configs").join("config.toml").to_string_lossy().to_string());
    if !std::path::Path::new(&config_path).exists() {
        startup::setup::generate_default_config(&config_path, &exe_dir)?;
    }
    unsafe { std::env::set_var("CONFIG_PATH", &config_path); }
    let config = AppConfig::load()?;

    // 初始化日志（控制台 + 文件双输出，大小轮转写入 wwwlogs/panel/）
    let log_dir = exe_dir.parent().and_then(|p| p.parent()).unwrap_or(&exe_dir).join("wwwlogs").join("panel");
    startup::logging::init(&log_dir, &config.log.level, config.log.max_size_mb);
    tracing::info!("配置加载完成");

    // 运行服务
    tracing::info!("启动 OxNginx 服务");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        // 初始化数据库
        tracing::info!("[1/4] 初始化数据库...");
        let db = Database::new(&config.database.path).await?;
        tracing::info!("[2/4] 数据库初始化完成");

        // 创建应用状态 & 构建路由
        tracing::info!("[3/4] 生成 RSA 密钥对...");
        let (rsa_private_key, rsa_public_key_b64) = crate::auth::generate_rsa_keypair()?;
        tracing::info!("[3/4] RSA 密钥对已生成");

        let state = AppState::new(db, config.clone(), rsa_private_key, rsa_public_key_b64);
        api::dashboard_ws::start_push_task(state.clone());

        // 启动操作日志后台批量写库 worker
        tracing::info!("[3.5/4] 启动操作日志 worker...");
        let audit_rx = crate::audit::sender::init();
        crate::audit::worker::spawn(audit_rx, state.db.pool().clone());
        tracing::info!("[3.5/4] 操作日志 worker 已启动");

        tracing::info!("[4/4] 构建路由...");
        let app = app::router::build(state);

        // 启动服务
        let addr = format!("{}:{}", config.server.host, config.server.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::info!("OxNginx 启动于 http://{}", addr);

        axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
