use std::path::Path;

/// 首次运行自动初始化
/// 检测 bundled 资源（nginx.zip、static/），自动解压、生成配置
pub fn first_run_setup(exe_dir: &Path) -> anyhow::Result<()> {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| exe_dir.join("configs").join("config.toml").to_string_lossy().to_string());

    // 已有配置文件则跳过
    if Path::new(&config_path).exists() {
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
pub fn generate_default_config(config_path: &str, exe_dir: &Path) -> anyhow::Result<()> {
    if let Some(parent) = Path::new(config_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_path = exe_dir.join("datas").join("data.db");
    let jwt_secret = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64_encode(&bytes)
    };

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

    for dir in &["datas", "wwwroot", "wwwlogs", "ssl", "backup"] {
        let _ = std::fs::create_dir_all(exe_dir.join(dir));
    }

    tracing::info!("默认配置已生成: {}", config_path);
    Ok(())
}
