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
    // 首次运行自动初始化（cargo-packager 安装后）
    let exe_dir = std::env::current_exe()?
        .parent().map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    first_run_setup(&exe_dir)?;

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

    // 加载配置（首次运行若无配置则自动生成默认配置）
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| exe_dir.join("configs").join("config.toml").to_string_lossy().to_string());
    if !std::path::Path::new(&config_path).exists() {
        tracing::info!("配置文件不存在，生成默认配置: {}", config_path);
        generate_default_config(&config_path, &exe_dir)?;
    }
    // 确保 AppConfig::load() 读取同一个文件
    unsafe { std::env::set_var("CONFIG_PATH", &config_path); }
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
            .route("/api/setup", post(api::auth_api::setup))
            .route("/api/setup/status", get(api::auth_api::setup_status));

        // 需要认证的路由
        let protected_routes = Router::new()
            .route("/api/dashboard", get(api::dashboard_api::get_dashboard))
            .route("/api/sites", get(api::site_api::list_sites))
            .route("/api/sites/with-certs", get(api::site_api::list_sites_with_certs))
            .route("/api/sites", post(api::site_api::create_site))
            .route("/api/sites/:id", get(api::site_api::get_site))
            .route("/api/sites/:id", put(api::site_api::update_site))
            .route("/api/sites/:id", delete(api::site_api::delete_site))
            .route("/api/sites/:id/deploy-ssl", post(api::site_api::deploy_ssl))
            .route("/api/sites/batch/enable", post(api::site_api::batch_enable))
            .route("/api/sites/batch/disable", post(api::site_api::batch_disable))
            .route("/api/sites/batch/delete", post(api::site_api::batch_delete))
            .route("/api/certificates", get(api::auth_api::list_certificates))
            .route("/api/certificate/apply", post(api::auth_api::apply_certificate))
            .route("/api/certificate/renew", post(api::auth_api::renew_certificate))
            .route("/api/change-password", post(api::auth_api::change_password))
            .route("/api/change-username", post(api::auth_api::change_username))
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
    })?;

    Ok(())
}

/// 首次运行自动初始化
/// 检测 bundled 资源（nginx.zip、static/），自动解压、生成配置
fn first_run_setup(exe_dir: &std::path::Path) -> anyhow::Result<()> {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| exe_dir.join("configs").join("config.toml").to_string_lossy().to_string());

    // 已有配置文件则跳过
    if std::path::Path::new(&config_path).exists() {
        return Ok(());
    }

    // 检查 bundled 资源是否存在
    let nginx_zip = exe_dir.join("libs").join("nginx").join("nginx-1.30.3.zip");
    let static_dir = exe_dir.join("static");
    tracing::info!("first_run_setup: exe_dir={}, nginx_zip={}, static_dir={}",
        exe_dir.display(), nginx_zip.exists(), static_dir.exists());
    if !nginx_zip.exists() || !static_dir.exists() {
        // 不是安装环境，正常启动（开发模式）
        return Ok(());
    }

    println!("");
    println!("  OxNginx 首次运行，正在初始化...");
    println!("  ========================================");

    // 创建目录结构
    let base = exe_dir; // C:\oxnginx\server\panel 或 /opt/oxnginx/server/panel
    let base_root = base.parent().and_then(|p| p.parent()).unwrap_or(base); // C:\oxnginx 或 /opt/oxnginx
    let dirs = [
        base.join("configs"),
        base.join("datas"),
        base_root.join("wwwroot"),
        base_root.join("wwwlogs"),
        base_root.join("ssl"),
        base_root.join("backup"),
        base_root.join("server").join("nginx"),
    ];
    for d in &dirs {
        std::fs::create_dir_all(d)?;
    }

    // 解压 nginx
    println!("  [1/3] 解压 nginx...");
    let nginx_target = base_root.join("server").join("nginx");
    let nginx_zip_file = std::fs::File::open(&nginx_zip)?;
    let mut archive = zip::ZipArchive::new(nginx_zip_file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = nginx_target.join(file.mangled_name());
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    // zip 内有 nginx-1.30.3/ 子目录，把内容移到上层
    let extracted_subdir = nginx_target.join("nginx-1.30.3");
    if extracted_subdir.exists() {
        for entry in std::fs::read_dir(&extracted_subdir)? {
            let entry = entry?;
            let dest = nginx_target.join(entry.file_name());
            // 目标已存在则跳过（配置文件等）
            if !dest.exists() {
                std::fs::rename(entry.path(), &dest)?;
            }
        }
        let _ = std::fs::remove_dir_all(&extracted_subdir);
    }
    println!("  [1/3] nginx 解压完成");

    // 生成 nginx.conf
    println!("  [2/3] 生成配置...");
    let nginx_conf = nginx_target.join("conf").join("nginx.conf");
    let sites_enabled = nginx_target.join("conf").join("sites-enabled");
    std::fs::create_dir_all(&sites_enabled)?;

    let wwwlogs = base_root.join("wwwlogs").to_string_lossy().replace('\\', "/");
    let se_path = sites_enabled.to_string_lossy().replace('\\', "/");
    std::fs::write(&nginx_conf, format!(
        "worker_processes 2;\nerror_log {wwwlogs}/error.log warn;\nevents {{ worker_connections 1024; }}\nhttp {{\n    include mime.types;\n    default_type application/octet-stream;\n    access_log {wwwlogs}/access.log;\n    sendfile on;\n    keepalive_timeout 65;\n    include {se_path}/*.conf;\n}}\n"
    ))?;

    // 生成 config.toml
    let jwt_secret = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64_encode(&bytes)
    };

    // Windows zip 里 nginx.exe 在根目录，Linux 编译的在 sbin/
    let nginx_bin = if cfg!(windows) {
        nginx_target.join("nginx.exe")
    } else {
        nginx_target.join("sbin").join("nginx")
    };
    let nginx_conf_path = nginx_target.join("conf").join("nginx.conf");
    let db_path = base.join("datas").join("data.db");
    let ssl_dir = base_root.join("ssl").to_string_lossy().replace('\\', "/");
    let wwwroot = base_root.join("wwwroot").to_string_lossy().replace('\\', "/");

    std::fs::write(&config_path, format!(
        r#"[server]
port = 9000
host = "0.0.0.0"

[database]
path = "{db}"

[nginx]
bin = "{bin}"
config = "{conf}"
sites_enabled = "{se}"
ssl_dir = "{ssl}"
default_root = "{root}"
log_access = "{logs}/access.log"
log_error = "{logs}/error.log"

[acme]
bin = ""
home = ""

[auth]
jwt_secret = "{jwt}"
jwt_expires_hours = 24
"#,
        db = db_path.to_string_lossy().replace('\\', "/"),
        bin = nginx_bin.to_string_lossy().replace('\\', "/"),
        conf = nginx_conf_path.to_string_lossy().replace('\\', "/"),
        se = sites_enabled.to_string_lossy().replace('\\', "/"),
        ssl = ssl_dir,
        root = wwwroot,
        logs = wwwlogs,
        jwt = jwt_secret,
    ))?;

    // 注册 Windows 服务
    #[cfg(target_os = "windows")]
    {
        let nssm = exe_dir.join("nssm.exe");
        if nssm.exists() {
            println!("  [3/3] 注册服务...");
            let svc_name = "OxNginx";
            let exe_path = exe_dir.join("ox-nginx.exe");
            let _ = std::process::Command::new(&nssm).args(["stop", svc_name]).output();
            let _ = std::process::Command::new(&nssm).args(["remove", svc_name, "confirm"]).output();
            let _ = std::process::Command::new(&nssm).args(["install", svc_name, exe_path.to_str().unwrap_or("")]).output();
            let _ = std::process::Command::new(&nssm).args(["set", svc_name, "AppDirectory", exe_dir.to_str().unwrap_or("")]).output();
            let _ = std::process::Command::new(&nssm).args(["set", svc_name, "DisplayName", "OxNginx"]).output();
            let _ = std::process::Command::new(&nssm).args(["set", svc_name, "Start", "SERVICE_AUTO_START"]).output();
            let env = format!("CONFIG_PATH={}", config_path);
            let _ = std::process::Command::new(&nssm).args(["set", svc_name, "AppEnvironmentExtra", &env, "RUST_LOG=info"]).output();
            let log = base_root.join("wwwlogs").join("panel.log").to_string_lossy().to_string();
            let _ = std::process::Command::new(&nssm).args(["set", svc_name, "AppStdout", &log]).output();
            let _ = std::process::Command::new(&nssm).args(["set", svc_name, "AppStderr", &log]).output();
            let _ = std::process::Command::new(&nssm).args(["set", svc_name, "AppRotateFiles", "1"]).output();
            let _ = std::process::Command::new(&nssm).args(["set", svc_name, "AppRotateBytes", "10485760"]).output();
            let _ = std::process::Command::new(&nssm).args(["start", svc_name]).output();
            println!("  [3/3] 服务已注册并启动");
        }
    }

    println!("  ========================================");
    println!("  初始化完成！");
    println!("  ========================================");
    println!("");

    Ok(())
}

/// 简单的 base64 编码
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

/// 生成默认配置文件（开发模式或无安装程序时使用）
fn generate_default_config(config_path: &str, exe_dir: &std::path::Path) -> anyhow::Result<()> {
    // 确保配置目录存在
    if let Some(parent) = std::path::Path::new(config_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_path = exe_dir.join("datas").join("data.db");
    let jwt_secret = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64_encode(&bytes)
    };

    // Windows 默认 nginx 路径（用户自行安装 nginx 后修改）
    #[cfg(target_os = "windows")]
    let (nginx_bin, nginx_conf, sites_enabled) = {
        let nginx_dir = exe_dir.join("nginx");
        (
            nginx_dir.join("nginx.exe").to_string_lossy().replace('\\', "/"),
            nginx_dir.join("conf").join("nginx.conf").to_string_lossy().replace('\\', "/"),
            nginx_dir.join("conf").join("sites-enabled").to_string_lossy().replace('\\', "/"),
        )
    };

    #[cfg(target_os = "linux")]
    let (nginx_bin, nginx_conf, sites_enabled) = (
        "/usr/sbin/nginx".to_string(),
        "/etc/nginx/nginx.conf".to_string(),
        "/etc/nginx/conf.d".to_string(),
    );

    std::fs::write(config_path, format!(
        r#"[server]
port = 9000
host = "0.0.0.0"

[database]
path = "{db}"

[nginx]
bin = "{bin}"
config = "{conf}"
sites_enabled = "{se}"
ssl_dir = "{base}/ssl"
default_root = "{base}/wwwroot"
log_access = "{base}/wwwlogs/access.log"
log_error = "{base}/wwwlogs/error.log"

[acme]
bin = ""
home = ""

[auth]
jwt_secret = "{jwt}"
jwt_expires_hours = 24
"#,
        db = db_path.to_string_lossy().replace('\\', "/"),
        bin = nginx_bin,
        conf = nginx_conf,
        se = sites_enabled,
        base = exe_dir.to_string_lossy().replace('\\', "/"),
        jwt = jwt_secret,
    ))?;

    // 创建基础目录
    for dir in &["datas", "wwwroot", "wwwlogs", "ssl", "backup"] {
        let _ = std::fs::create_dir_all(exe_dir.join(dir));
    }

    tracing::info!("默认配置已生成: {}", config_path);
    Ok(())
}
