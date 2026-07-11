use std::path::Path;

/// 获取配置文件路径
fn get_config_path(exe_dir: &Path) -> String {
    std::env::var("CONFIG_PATH").unwrap_or_else(|_| {
        exe_dir
            .join("configs")
            .join("config.toml")
            .to_string_lossy()
            .to_string()
    })
}

/// 创建基础目录（install 和 dev 都要）
fn create_base_dirs(exe_dir: &Path) -> std::io::Result<()> {
    let dirs = [
        exe_dir.join("configs"),
        exe_dir.join("datas"),
        exe_dir.join("wwwroot"),
        exe_dir.join("wwwlogs").join("nginx"),
        exe_dir.join("wwwlogs").join("panel"),
        exe_dir.join("ssl"),
        exe_dir.join("backup"),
        exe_dir.join("server").join("nginx"),
    ];
    for d in &dirs {
        std::fs::create_dir_all(d)?;
    }
    Ok(())
}

/// 生成 config.toml 内容
fn build_config_content(exe_dir: &Path) -> String {
    let db_path = exe_dir.join("datas").join("data.db");
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
token_expires_hours = 1

[log]
level = "info"
max_size_mb = 10
"#,
        db = db_path.to_string_lossy().replace('\\', "/"),
    )
}

/// 首次运行自动初始化：创建基础目录 + 生成 config.toml
pub fn first_run_setup(exe_dir: &Path) -> anyhow::Result<()> {
    println!("[setup.rs:first_run_setup] ===> 执行首次运行初始化");

    let config_path = get_config_path(exe_dir);
    println!(
        "[setup.rs:first_run_setup] ===> 配置文件路径: {}",
        config_path
    );

    // 已有配置文件则跳过
    if Path::new(&config_path).exists() {
        println!("[setup.rs:first_run_setup] ===> 配置文件已存在，跳过初始化");
        return Ok(());
    }

    // 创建基础目录
    create_base_dirs(exe_dir)?;
    println!("[setup.rs:first_run_setup] ===> 基础目录创建完成");

    // 生成 config.toml
    std::fs::write(&config_path, build_config_content(exe_dir))?;
    println!("[setup.rs:first_run_setup] ===> config.toml 已生成");
    println!("[setup.rs:first_run_setup] ===> 初始化完成");

    Ok(())
}

/// 生成默认配置文件（开发模式或无安装程序时使用）
pub fn generate_default_config(config_path: &str, exe_dir: &Path) -> anyhow::Result<()> {
    if let Some(parent) = Path::new(config_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(config_path, build_config_content(exe_dir))?;
    println!(
        "[setup.rs:generate_default_config] ===> 默认配置已生成: {}",
        config_path
    );

    // 开发模式基础目录（与 first_run_setup 互补）
    let dirs = [
        "datas",
        "wwwroot",
        "wwwlogs/nginx",
        "wwwlogs/panel",
        "ssl",
        "backup",
        "server",
    ];
    for d in &dirs {
        std::fs::create_dir_all(exe_dir.join(d))?;
    }

    Ok(())
}
