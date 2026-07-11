use std::path::Path;

/// 首次运行自动初始化
/// 检测 bundled 资源（nginx.zip、static/），自动解压、生成配置
pub fn first_run_setup(exe_dir: &Path) -> anyhow::Result<()> {
    println!("[setup.rs:first_run_setup] ===> 执行首次运行初始化");
    // 尝试从环境变量 "CONFIG_PATH" 获取配置文件的路径
    //  ↓ 如果环境变量没设置
    //  就用默认值: {exe程序所在目录}/configs/config.toml
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| {
        exe_dir
            .join("configs")
            .join("config.toml")
            .to_string_lossy()
            .to_string()
    });
    println!(
        "[setup.rs:first_run_setup] ===> 配置文件路径: {}",
        config_path
    );

    // 已有配置文件则跳过
    if Path::new(&config_path).exists() {
        println!("[setup.rs:first_run_setup] ===> 配置文件已存在，跳过初始化");
        return Ok(());
    }

    // 检查 bundled 资源是否存在
    let nginx_zip = exe_dir.join("libs").join("nginx").join("nginx-1.30.3.zip");
    let static_dir = exe_dir.join("static");
    println!(
        "[setup.rs:first_run_setup] ===> exe_dir={}, nginx_zip={}, static_dir={}",
        exe_dir.display(),
        nginx_zip.exists(),
        static_dir.exists()
    );
    if !nginx_zip.exists() || !static_dir.exists() {
        // 不是安装环境，正常启动（开发模式）
        println!("[setup.rs:first_run_setup] ===> 不是安装环境，正常启动（开发模式）");
        return Ok(());
    }

    println!("");
    println!("[setup.rs:first_run_setup] ===> OxNginx 首次运行，正在初始化...");
    println!("[setup.rs:first_run_setup] ========================================");

    // 创建目录结构
    let base = exe_dir; // C:\oxnginx\server\panel 或 /opt/oxnginx/server/panel
    let base_root = base.parent().and_then(|p| p.parent()).unwrap_or(base); // C:\oxnginx 或 /opt/oxnginx
    println!(
        "[setup.rs:first_run_setup] 创建目录结构 base={}, base_root={}",
        base.display(),
        base_root.display()
    );
    let dirs = [
        base.join("configs"),
        base.join("datas"),
        base_root.join("wwwroot"),
        base_root.join("wwwlogs").join("nginx"),
        base_root.join("wwwlogs").join("panel"),
        base_root.join("ssl"),
        base_root.join("backup"),
        base_root.join("server").join("nginx"),
    ];
    println!("[setup.rs:first_run_setup] ===> 将创建以下目录:");
    for d in &dirs {
        println!("[setup.rs:first_run_setup] ===> 创建目录: {}", d.display());
        std::fs::create_dir_all(d)?;
    }

    // 解压 nginx
    println!("[setup.rs:first_run_setup] ===> [1/3] 解压 nginx...");
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
    println!("[setup.rs:first_run_setup] ===> [1/3] nginx 解压完成");

    // 生成 nginx.conf
    println!("[setup.rs:first_run_setup] ===> [2/3] 生成配置...");
    let nginx_conf = nginx_target.join("conf").join("nginx.conf");
    let sites_enabled = nginx_target.join("conf").join("sites-enabled");
    std::fs::create_dir_all(&sites_enabled)?;

    let nginx_logs = base_root
        .join("wwwlogs")
        .join("nginx")
        .to_string_lossy()
        .replace('\\', "/");
    let se_path = sites_enabled.to_string_lossy().replace('\\', "/");
    std::fs::write(&nginx_conf, format!(
        "worker_processes 2;\nerror_log {nginx_logs}/error.log warn;\nevents {{ worker_connections 1024; }}\nhttp {{\n    include mime.types;\n    default_type application/octet-stream;\n    access_log {nginx_logs}/access.log;\n    sendfile on;\n    keepalive_timeout 65;\n    include {se_path}/*.conf;\n}}\n"
    ))?;

    // 生成 config.toml
    println!("[setup.rs:first_run_setup] ===> [3/3] 生成配置文件...");
    let db_path = base.join("datas").join("data.db");

    std::fs::write(
        &config_path,
        format!(
            r#"[server]
port = 9000
host = "0.0.0.0"

[database]
path = "{db}"
log_sql = false

[acme]
bin = ""
home = ""

[auth]
token_expires_hours = 24

[log]
level = "debug"
max_size_mb = 10
"#,
            db = db_path.to_string_lossy().replace('\\', "/"),
        ),
    )?;

    // 注册 Windows 服务
    #[cfg(target_os = "windows")]
    {
        let nssm = exe_dir.join("nssm.exe");
        if nssm.exists() {
            println!("[setup.rs:first_run_setup] ===> [3/3] 注册服务...");
            let svc_name = "OxNginx";
            let exe_path = exe_dir.join("ox-nginx.exe");

            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            let run_nssm = |args: &[&str]| {
                let _ = std::process::Command::new(&nssm)
                    .args(args)
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            };

            run_nssm(&["stop", svc_name]);
            run_nssm(&["remove", svc_name, "confirm"]);
            run_nssm(&["install", svc_name, exe_path.to_str().unwrap_or("")]);
            run_nssm(&[
                "set",
                svc_name,
                "AppDirectory",
                exe_dir.to_str().unwrap_or(""),
            ]);
            run_nssm(&["set", svc_name, "DisplayName", "OxNginx"]);
            run_nssm(&["set", svc_name, "Start", "SERVICE_AUTO_START"]);
            let env = format!("CONFIG_PATH={}", config_path);
            run_nssm(&[
                "set",
                svc_name,
                "AppEnvironmentExtra",
                &env,
                "RUST_LOG=info",
            ]);
            let log = base_root
                .join("wwwlogs")
                .join("panel")
                .join("nssm.log")
                .to_string_lossy()
                .to_string();
            run_nssm(&["set", svc_name, "AppStdout", &log]);
            run_nssm(&["set", svc_name, "AppStderr", &log]);
            run_nssm(&["set", svc_name, "AppRotateFiles", "1"]);
            run_nssm(&["set", svc_name, "AppRotateBytes", "10485760"]);
            run_nssm(&["start", svc_name]);
            println!("[setup.rs:first_run_setup] ===> [3/3] 服务已注册并启动");
        }
    }

    println!("[setup.rs:first_run_setup] ===> ========================================");
    println!("[setup.rs:first_run_setup] ===> 初始化完成！");
    println!("[setup.rs:first_run_setup] ===> ========================================");

    Ok(())
}

/// 生成默认配置文件（开发模式或无安装程序时使用）
pub fn generate_default_config(config_path: &str, exe_dir: &Path) -> anyhow::Result<()> {
    if let Some(parent) = Path::new(config_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_path = exe_dir.join("datas").join("data.db");

    std::fs::write(
        config_path,
        format!(
            r#"[server]
port = 9000
host = "0.0.0.0"

[database]
path = "{db}"
log_sql = false

[acme]
bin = ""
home = ""

[auth]
token_expires_hours = 24

[log]
level = "debug"
max_size_mb = 10
"#,
            db = db_path.to_string_lossy().replace('\\', "/"),
        ),
    )?;

    for dir in &[
        "datas",
        "wwwroot",
        "wwwlogs/nginx",
        "wwwlogs/panel",
        "ssl",
        "backup",
        "server",
    ] {
        let _ = std::fs::create_dir_all(exe_dir.join(dir));
    }

    println!(
        "[setup.rs:generate_default_config] ===> 默认配置已生成: {}",
        config_path
    );
    Ok(())
}
