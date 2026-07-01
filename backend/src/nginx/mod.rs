use crate::dto::{CreateUpstreamRequest, NginxTestResult};
use crate::model::{Site, Upstream, UpstreamServer};

/// 生成Nginx站点配置
pub fn generate_site_config(site: &Site) -> String {
    let mut config = String::new();

    // server块开始
    config.push_str("server {\n");

    // listen
    if site.ssl == 1 {
        config.push_str(&format!("    listen {} ssl;\n", site.listen));
        config.push_str("    listen [::]:443 ssl;\n");
    } else {
        config.push_str(&format!("    listen {};\n", site.listen));
        config.push_str(&format!("    listen [::]:{};\n", site.listen));
    }

    // server_name
    config.push_str(&format!("    server_name {};\n", site.server_name));

    // SSL配置
    if site.ssl == 1 {
        if let Some(cert_path) = &site.certificate_path {
            config.push_str(&format!("    ssl_certificate {};\n", cert_path));
        }
        if let Some(key_path) = &site.key_path {
            config.push_str(&format!("    ssl_certificate_key {};\n", key_path));
        }
        config.push_str("    ssl_protocols TLSv1.2 TLSv1.3;\n");
        config.push_str("    ssl_ciphers HIGH:!aNULL:!MD5;\n");
    }

    // 反向代理或静态文件
    if let Some(proxy_pass) = &site.proxy_pass {
        config.push_str("\n    location / {\n");
        config.push_str(&format!("        proxy_pass {};\n", proxy_pass));
        config.push_str("        proxy_set_header Host $host;\n");
        config.push_str("        proxy_set_header X-Real-IP $remote_addr;\n");
        config.push_str("        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
        config.push_str("        proxy_set_header X-Forwarded-Proto $scheme;\n");

        // WebSocket支持
        config.push_str("        proxy_http_version 1.1;\n");
        config.push_str("        proxy_set_header Upgrade $http_upgrade;\n");
        config.push_str("        proxy_set_header Connection \"upgrade\";\n");
        config.push_str("    }\n");
    } else if let Some(root_path) = &site.root_path {
        // 统一使用正斜杠，避免 Windows 反斜杠导致 nginx 解析失败
        let root_path = root_path.replace('\\', "/");
        config.push_str(&format!("\n    root {};\n", root_path));
        config.push_str("    index index.html index.htm;\n");
        config.push_str("\n    location / {\n");
        config.push_str("        try_files $uri $uri/ /index.html;\n");
        config.push_str("    }\n");
    }

    config.push_str("}\n");

    config
}

/// 测试Nginx配置
pub async fn test_config(nginx_bin: &str) -> NginxTestResult {
    use tokio::process::Command;

    let nginx_dir = match get_nginx_dir(nginx_bin) {
        Some(dir) => dir,
        None => return NginxTestResult {
            success: false,
            message: "无法获取 nginx 安装目录".to_string(),
        },
    };

    let output = Command::new(nginx_bin)
        .current_dir(&nginx_dir)
        .arg("-t")
        .output()
        .await;

    match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            NginxTestResult {
                success: out.status.success(),
                message: stderr.to_string(),
            }
        }
        Err(e) => NginxTestResult {
            success: false,
            message: format!("执行nginx命令失败: {}", e),
        },
    }
}

/// 写入站点配置文件
pub async fn write_site_config(
    sites_enabled: &str,
    site_name: &str,
    config: &str,
) -> anyhow::Result<()> {
    tracing::info!("write_site_config: sites_enabled={}, site_name={}", sites_enabled, site_name);
    // 确保 sites-enabled 目录存在
    tokio::fs::create_dir_all(sites_enabled).await?;
    let config_path = format!("{}/{}.conf", sites_enabled, site_name);
    tracing::info!("write_site_config: config_path={}", config_path);
    tokio::fs::write(&config_path, config).await?;
    Ok(())
}

/// 删除站点配置文件
pub async fn remove_site_config(sites_enabled: &str, site_name: &str) -> anyhow::Result<()> {
    let config_path = format!("{}/{}.conf", sites_enabled, site_name);
    if tokio::fs::metadata(&config_path).await.is_ok() {
        tokio::fs::remove_file(&config_path).await?;
    }
    Ok(())
}

/// 确保 nginx.conf 中包含 sites-enabled 目录的 include 指令
pub async fn ensure_sites_enabled_include(nginx_config: &str, sites_enabled: &str) -> anyhow::Result<()> {
    let content = tokio::fs::read_to_string(nginx_config).await?;
    // 检查是否已包含 sites-enabled 的 include
    if content.contains("sites-enabled") {
        return Ok(());
    }
    // 在 http 块的最后一个 } 前插入 include 指令
    // 使用正斜杠路径
    let sites_path = sites_enabled.replace('\\', "/");
    let include_line = format!("\n    include {}/*.conf;\n", sites_path);
    // 找到最后一个 } (http块的结束大括号)
    if let Some(pos) = content.rfind('}') {
        let mut new_content = content[..pos].to_string();
        new_content.push_str(&include_line);
        new_content.push_str("}\n");
        tokio::fs::write(nginx_config, new_content).await?;
    }
    Ok(())
}

/// 创建默认 index.html
pub async fn create_default_index(root_path: &str) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(root_path).await?;
    let index_path = format!("{}/index.html", root_path);
    let content = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>站点创建成功</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; display: flex; justify-content: center; align-items: center; min-height: 100vh; margin: 0; background: #f5f5f5; }
        .card { background: white; padding: 40px; border-radius: 8px; box-shadow: 0 2px 12px rgba(0,0,0,0.1); text-align: center; }
        h1 { color: #67c23a; margin-bottom: 10px; }
        p { color: #909399; }
    </style>
</head>
<body>
    <div class="card">
        <h1>🎉 恭喜, 站点创建成功！</h1>
        <p>这是默认 index.html，本页面由系统自动生成</p>
    </div>
</body>
</html>"#;
    tokio::fs::write(&index_path, content).await?;
    Ok(())
}

/// Nginx 进程状态
#[derive(Debug, serde::Serialize)]
pub struct NginxStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub version: Option<String>,
    pub uptime: Option<String>,
    #[serde(default)]
    pub not_installed: bool,
}

/// 获取 nginx 可执行文件所在目录
fn get_nginx_dir(nginx_bin: &str) -> Option<std::path::PathBuf> {
    std::path::Path::new(nginx_bin).parent().map(|p| p.to_path_buf())
}

/// 获取 Nginx 运行状态
pub async fn get_nginx_status(nginx_bin: &str) -> NginxStatus {
    use tokio::process::Command;
    use std::env::consts::OS;

    tracing::debug!("[NginxStatus] 检测 nginx: bin={}, os={}", nginx_bin, OS);

    // 先检测 nginx 是否可执行
    let version_check = Command::new(nginx_bin).arg("-v").output().await;
    let not_installed = version_check.is_err();

    tracing::debug!("[NginxStatus] not_installed={}, version_check_err={}", not_installed, version_check.is_err());

    if not_installed {
        return NginxStatus {
            running: false,
            pid: None,
            version: None,
            uptime: None,
            not_installed: true,
        };
    }

    // 获取版本
    let version = match &version_check {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            stderr.lines()
                .find(|l| l.contains("version"))
                .map(|l| l.trim().to_string())
        }
        _ => None,
    };

    // 检查进程是否存在（跨平台）
    let (pid, running) = if OS == "windows" {
        // Windows: 使用 tasklist 查找 nginx 进程
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq nginx.exe", "/FO", "CSV", "/NH"])
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // CSV 格式: "nginx.exe","12345","Console","1","10,000 K"
                let pid = stdout.lines()
                    .find(|l| l.contains("nginx.exe"))
                    .and_then(|l| {
                        l.split(',').nth(1).and_then(|s| {
                            s.trim_matches('"').parse::<u32>().ok()
                        })
                    });
                (pid, pid.is_some())
            }
            _ => (None, false),
        }
    } else {
        // Linux: 使用 pgrep
        let output = Command::new("pgrep")
            .args(["-x", "nginx"])
            .output()
            .await;

        let pid = match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.lines().next().and_then(|p| p.trim().parse::<u32>().ok())
            }
            _ => None,
        };
        (pid, pid.is_some())
    };

    // 获取运行时间（跨平台）
    let uptime = if let Some(pid) = pid {
        if OS == "windows" {
            // Windows: 通过 wmic 获取进程启动时间
            let output = Command::new("wmic")
                .args(["process", "where", &format!("ProcessId={}", pid), "get", "CreationDate", "/value"])
                .output()
                .await;
            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    // 解析 CreationDate 并计算运行时间
                    stdout.lines()
                        .find(|l| l.starts_with("CreationDate="))
                        .and_then(|l| {
                            let date_str = l.strip_prefix("CreationDate=")?.trim();
                            if date_str.len() >= 14 {
                                // 格式: 20260701120000.000000+480
                                Some(format!("{}-{}-{} {}:{}:{}",
                                    &date_str[0..4], &date_str[4..6], &date_str[6..8],
                                    &date_str[8..10], &date_str[10..12], &date_str[12..14]))
                            } else {
                                None
                            }
                        })
                }
                _ => None,
            }
        } else {
            let output = Command::new("ps")
                .args(["-o", "etime=", "-p", &pid.to_string()])
                .output()
                .await;
            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    Some(stdout.trim().to_string())
                }
                _ => None,
            }
        }
    } else {
        None
    };

    NginxStatus {
        running,
        pid,
        version,
        uptime,
        not_installed: false,
    }
}

/// 启动 Nginx
pub async fn start_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    use tokio::process::Command;

    // nginx 需要从其安装目录运行
    let nginx_dir = get_nginx_dir(nginx_bin)
        .ok_or_else(|| anyhow::anyhow!("无法获取 nginx 安装目录"))?;

    tracing::info!("启动 Nginx: bin={}, dir={}", nginx_bin, nginx_dir.display());

    // 使用 spawn 启动，不等待进程结束（nginx 是守护进程）
    let child = Command::new(nginx_bin)
        .current_dir(&nginx_dir)
        .spawn();

    match child {
        Ok(_child) => {
            // 等待一小段时间让 nginx 完成启动
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            // 检查 nginx 是否正在运行
            let status = get_nginx_status(nginx_bin).await;
            Ok(status.running)
        }
        Err(e) => {
            tracing::error!("启动 Nginx 失败: {}", e);
            Err(anyhow::anyhow!("启动 Nginx 失败: {}", e))
        }
    }
}

/// 停止 Nginx
pub async fn stop_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    use tokio::process::Command;

    let nginx_dir = get_nginx_dir(nginx_bin)
        .ok_or_else(|| anyhow::anyhow!("无法获取 nginx 安装目录"))?;

    tracing::info!("停止 Nginx: bin={}", nginx_bin);

    let output = Command::new(nginx_bin)
        .current_dir(&nginx_dir)
        .args(["-s", "stop"])
        .output()
        .await?;

    Ok(output.status.success())
}

/// 重载 Nginx 配置
pub async fn reload_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    use tokio::process::Command;

    let nginx_dir = get_nginx_dir(nginx_bin)
        .ok_or_else(|| anyhow::anyhow!("无法获取 nginx 安装目录"))?;

    tracing::info!("重载 Nginx 配置: bin={}", nginx_bin);

    let output = Command::new(nginx_bin)
        .current_dir(&nginx_dir)
        .args(["-s", "reload"])
        .output()
        .await?;

    Ok(output.status.success())
}

/// 重启 Nginx
pub async fn restart_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    // 先停止
    let _ = stop_nginx(nginx_bin).await;
    // 等待一小段时间
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    // 再启动
    start_nginx(nginx_bin).await
}

/// 一键安装 Nginx（Windows/Linux）
pub async fn install_nginx(install_dir: &str) -> anyhow::Result<NginxInstallResult> {
    use tokio::process::Command;
    use std::env::consts::OS;

    let os = OS;

    if os == "windows" {
        // Windows: 下载 nginx 并解压到安装目录
        let nginx_version = "1.30.3";
        let download_url = format!(
            "https://nginx.org/download/nginx-{}.zip",
            nginx_version
        );
        let zip_path = format!("{}\\nginx-{}.zip", install_dir, nginx_version);

        // 创建安装目录
        tokio::fs::create_dir_all(install_dir).await?;

        // 下载
        tracing::info!("下载 Nginx {}...", nginx_version);
        let output = Command::new("curl")
            .args(["-L", "-o", &zip_path, &download_url])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("下载 Nginx 失败"));
        }

        // 解压 - Windows 用 PowerShell Expand-Archive
        tracing::info!("解压 Nginx...");
        let ps_cmd = format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
            zip_path, install_dir
        );
        let output = Command::new("powershell")
            .args(["-Command", &ps_cmd])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("解压 Nginx 失败"));
        }

        // 清理 zip
        let _ = tokio::fs::remove_file(&zip_path).await;

        // 返回 nginx.exe 路径和配置路径
        let nginx_exe = format!("{}\\nginx-{}\\nginx.exe", install_dir, nginx_version);
        let nginx_conf = format!("{}\\nginx-{}\\conf\\nginx.conf", install_dir, nginx_version);
        let sites_enabled = format!("{}\\nginx-{}\\conf\\sites-enabled", install_dir, nginx_version);

        tracing::info!("Nginx 安装完成: {}", nginx_exe);
        Ok(NginxInstallResult {
            bin: nginx_exe,
            config: nginx_conf,
            sites_enabled,
        })
    } else {
        // Linux: 使用包管理器
        let install_cmd = if Command::new("which").arg("apt-get").output().await.map(|o| o.status.success()).unwrap_or(false) {
            "apt-get install -y nginx"
        } else if Command::new("which").arg("yum").output().await.map(|o| o.status.success()).unwrap_or(false) {
            "yum install -y nginx"
        } else if Command::new("which").arg("dnf").output().await.map(|o| o.status.success()).unwrap_or(false) {
            "dnf install -y nginx"
        } else {
            return Err(anyhow::anyhow!("未找到支持的包管理器"));
        };

        tracing::info!("安装 Nginx...");
        let output = Command::new("sh")
            .args(["-c", install_cmd])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("安装 Nginx 失败"));
        }

        // 返回 nginx 路径
        let nginx_path = Command::new("which")
            .arg("nginx")
            .output()
            .await?;

        let bin = String::from_utf8_lossy(&nginx_path.stdout).trim().to_string();
        tracing::info!("Nginx 安装完成: {}", bin);
        Ok(NginxInstallResult {
            bin,
            config: "/etc/nginx/nginx.conf".to_string(),
            sites_enabled: "/etc/nginx/sites-enabled".to_string(),
        })
    }
}

/// Nginx 安装结果
#[derive(Debug)]
pub struct NginxInstallResult {
    pub bin: String,
    pub config: String,
    pub sites_enabled: String,
}

/// 生成上游服务器配置
pub fn generate_upstream_config(upstream: &Upstream, servers: &[UpstreamServer]) -> String {
    let mut config = String::new();

    config.push_str(&format!("upstream {} {{\n", upstream.name));

    // 负载均衡方法
    match upstream.method.as_str() {
        "ip_hash" => config.push_str("    ip_hash;\n"),
        "least_conn" => config.push_str("    least_conn;\n"),
        "hash" => config.push_str(&format!("    hash $request_uri;\n")),
        _ => {} // round_robin 是默认方法，不需要特殊指令
    }

    // keepalive
    if upstream.keepalive > 0 {
        config.push_str(&format!("    keepalive {};\n", upstream.keepalive));
    }

    // 服务器节点
    for server in servers {
        if server.status != "enabled" {
            continue;
        }

        config.push_str(&format!("    server {}", server.address));

        if server.weight != 1 {
            config.push_str(&format!(" weight={}", server.weight));
        }

        if server.max_fails != 3 {
            config.push_str(&format!(" max_fails={}", server.max_fails));
        }

        if server.fail_timeout != "30s" {
            config.push_str(&format!(" fail_timeout={}", server.fail_timeout));
        }

        if server.backup == 1 {
            config.push_str(" backup");
        }

        config.push_str(";\n");
    }

    config.push_str("}\n");

    config
}

/// 从请求生成上游服务器配置
pub fn generate_upstream_config_from_request(req: &CreateUpstreamRequest) -> String {
    let mut config = String::new();

    config.push_str(&format!("upstream {} {{\n", req.name));

    // 负载均衡方法
    match req.method.as_str() {
        "ip_hash" => config.push_str("    ip_hash;\n"),
        "least_conn" => config.push_str("    least_conn;\n"),
        "hash" => config.push_str(&format!("    hash $request_uri;\n")),
        _ => {} // round_robin 是默认方法，不需要特殊指令
    }

    // keepalive
    if req.keepalive > 0 {
        config.push_str(&format!("    keepalive {};\n", req.keepalive));
    }

    // 服务器节点
    for server in &req.servers {
        config.push_str(&format!("    server {}", server.address));

        if server.weight != 1 {
            config.push_str(&format!(" weight={}", server.weight));
        }

        if server.max_fails != 3 {
            config.push_str(&format!(" max_fails={}", server.max_fails));
        }

        if server.fail_timeout != "30s" {
            config.push_str(&format!(" fail_timeout={}", server.fail_timeout));
        }

        if server.backup {
            config.push_str(" backup");
        }

        config.push_str(";\n");
    }

    config.push_str("}\n");

    config
}
