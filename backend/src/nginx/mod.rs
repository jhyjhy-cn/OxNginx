use crate::dto::{CreateUpstreamRequest, NginxTestResult};
use crate::model::{Site, Upstream, UpstreamServer};
use serde::Deserialize;

/// 重定向规则
#[derive(Debug, Deserialize)]
struct RedirectRule {
    domain: String,
    target: String,
    #[serde(default)]
    redirect_type: u16, // 301 or 302
}

/// 防盗链配置
#[derive(Debug, Deserialize)]
struct HotlinkConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    allowed_domains: Vec<String>,
    #[serde(default = "default_hotlink_code")]
    return_code: u16,
}

fn default_hotlink_code() -> u16 { 403 }

/// 解析 server_name 中的 host:port，返回 (纯 host 列表, 额外端口列表)
/// 例如 "test.com 192.168.3.14:81 192.168.3.14:82" → (["test.com","192.168.3.14"], [81,82])
fn parse_server_names(server_name: &str) -> (Vec<String>, Vec<u16>) {
    let mut hosts = Vec::new();
    let mut ports = Vec::new();
    for part in server_name.split_whitespace() {
        if let Some(idx) = part.rfind(':') {
            let host = &part[..idx];
            let port_str = &part[idx + 1..];
            if let Ok(port) = port_str.parse::<u16>() {
                hosts.push(host.to_string());
                if !ports.contains(&port) {
                    ports.push(port);
                }
            } else {
                hosts.push(part.to_string());
            }
        } else {
            hosts.push(part.to_string());
        }
    }
    (hosts, ports)
}

/// 追加公共 server 指令（日志、rewrite、防盗链、重定向）
fn append_common_directives(config: &mut String, site: &Site, indent: &str) {
    // per-site 日志
    if let Some(ref path) = site.log_access_path {
        config.push_str(&format!("{}access_log {};\n", indent, path));
    }
    if let Some(ref path) = site.log_error_path {
        config.push_str(&format!("{}error_log {};\n", indent, path));
    }

    // 伪静态 rewrite 规则
    if let Some(ref json) = site.rewrite_rules {
        if let Ok(rules) = serde_json::from_str::<Vec<serde_json::Value>>(json) {
            for rule in &rules {
                let pattern = rule["pattern"].as_str().unwrap_or("");
                let replacement = rule["replacement"].as_str().unwrap_or("");
                let flag = rule["flag"].as_str().unwrap_or("last");
                if !pattern.is_empty() {
                    config.push_str(&format!("{}rewrite {} {} {};\n", indent, pattern, replacement, flag));
                }
            }
        }
    }

    // 防盗链
    if let Some(ref json) = site.hotlink_config {
        if let Ok(hc) = serde_json::from_str::<HotlinkConfig>(json) {
            if hc.enabled && !hc.allowed_domains.is_empty() {
                config.push_str(&format!("\n{}valid_referers none blocked", indent));
                for domain in &hc.allowed_domains {
                    config.push_str(&format!(" {}", domain));
                }
                config.push_str(";\n");
                config.push_str(&format!("{}if ($invalid_referer) {{\n", indent));
                config.push_str(&format!("{}    return {};\n", indent, hc.return_code));
                config.push_str(&format!("{}}}\n", indent));
            }
        }
    }

    // 重定向规则（作为额外 location 块）
    if let Some(ref json) = site.redirect_rules {
        if let Ok(rules) = serde_json::from_str::<Vec<RedirectRule>>(json) {
            for rule in &rules {
                let code = if rule.redirect_type == 302 { 302 } else { 301 };
                config.push_str(&format!("\n{}location {} {{\n", indent, rule.domain));
                config.push_str(&format!("{}    return {} {};\n", indent, code, rule.target));
                config.push_str(&format!("{}}}\n", indent));
            }
        }
    }
}

/// 生成Nginx站点配置
pub fn generate_site_config(site: &Site) -> String {
    let mut config = String::new();

    if site.ssl == 1 {
        let (hosts, extra_ports) = parse_server_names(&site.server_name);
        let server_name_clean = hosts.join(" ");
        let main_port: u16 = site.listen.parse().unwrap_or(80);

        // 额外的 HTTP 端口（非 80、非 443）也需要生成跳转块
        let mut redirect_ports = vec![main_port];
        for p in &extra_ports {
            if *p != 443 && !redirect_ports.contains(p) {
                redirect_ports.push(*p);
            }
        }

        // ========== HTTP 端口：强制跳转到 HTTPS ==========
        for port in &redirect_ports {
            config.push_str("server {\n");
            config.push_str(&format!("    listen {};\n", port));
            config.push_str(&format!("    listen [::]:{};\n", port));
            config.push_str(&format!("    server_name {};\n", server_name_clean));
            config.push_str("    return 301 https://$host$request_uri;\n");
            config.push_str("}\n\n");
        }

        // ========== 443 端口：SSL 终止 ==========
        let mut ssl_ports = vec![443u16];
        for p in &extra_ports {
            if *p != 443 && *p != main_port && !redirect_ports.contains(p) && !ssl_ports.contains(p) {
                ssl_ports.push(*p);
            }
        }

        for port in &ssl_ports {
            config.push_str("server {\n");
            config.push_str(&format!("    listen {} ssl;\n", port));
            config.push_str(&format!("    listen [::]:{} ssl;\n", port));
            config.push_str(&format!("    server_name {};\n", server_name_clean));
            if let Some(cert_path) = &site.certificate_path {
                config.push_str(&format!("    ssl_certificate {};\n", cert_path));
            }
            if let Some(key_path) = &site.key_path {
                config.push_str(&format!("    ssl_certificate_key {};\n", key_path));
            }
            config.push_str("    ssl_protocols TLSv1.2 TLSv1.3;\n");
            config.push_str("    ssl_ciphers HIGH:!aNULL:!MD5;\n");

            append_common_directives(&mut config, site, "    ");

            if let Some(proxy_pass) = &site.proxy_pass {
                config.push_str("\n    location / {\n");
                config.push_str(&format!("        proxy_pass {};\n", proxy_pass));
                config.push_str("        proxy_set_header Host $host;\n");
                config.push_str("        proxy_set_header X-Real-IP $remote_addr;\n");
                config.push_str("        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
                config.push_str("        proxy_set_header X-Forwarded-Proto $scheme;\n");
                config.push_str("        proxy_http_version 1.1;\n");
                config.push_str("        proxy_set_header Upgrade $http_upgrade;\n");
                config.push_str("        proxy_set_header Connection \"upgrade\";\n");
                config.push_str("    }\n");
            } else if let Some(root_path) = &site.root_path {
                let root_path = root_path.replace('\\', "/");
                config.push_str(&format!("\n    root {};\n", root_path));
                config.push_str("    index index.html index.htm;\n");
                config.push_str("\n    location = /index.html {\n");
                config.push_str("        expires -1;\n");
                config.push_str("    }\n");
                config.push_str("\n    location / {\n");
                config.push_str("        try_files $uri $uri/ =404;\n");
                config.push_str("    }\n");
            }
            config.push_str("}\n\n");
        }
    } else {
        // 非 SSL：普通 HTTP server
        let (hosts, extra_ports) = parse_server_names(&site.server_name);
        let server_name_clean = hosts.join(" ");
        // 收集所有监听端口（主端口 + server_name 中的端口，去重）
        let main_port: u16 = site.listen.parse().unwrap_or(80);
        let mut all_ports = vec![main_port];
        for p in &extra_ports {
            if !all_ports.contains(p) {
                all_ports.push(*p);
            }
        }

        for port in &all_ports {
            config.push_str("server {\n");
            config.push_str(&format!("    listen {};\n", port));
            config.push_str(&format!("    listen [::]:{};\n", port));
            config.push_str(&format!("    server_name {};\n", server_name_clean));

            append_common_directives(&mut config, site, "    ");

            if let Some(proxy_pass) = &site.proxy_pass {
                config.push_str("\n    location / {\n");
                config.push_str(&format!("        proxy_pass {};\n", proxy_pass));
                config.push_str("        proxy_set_header Host $host;\n");
                config.push_str("        proxy_set_header X-Real-IP $remote_addr;\n");
                config.push_str("        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
                config.push_str("        proxy_set_header X-Forwarded-Proto $scheme;\n");
                config.push_str("        proxy_http_version 1.1;\n");
                config.push_str("        proxy_set_header Upgrade $http_upgrade;\n");
                config.push_str("        proxy_set_header Connection \"upgrade\";\n");
                config.push_str("    }\n");
            } else if let Some(root_path) = &site.root_path {
                let root_path = root_path.replace('\\', "/");
                config.push_str(&format!("\n    root {};\n", root_path));
                config.push_str("    index index.html index.htm;\n");
                config.push_str("\n    location = /index.html {\n");
                config.push_str("        expires -1;\n");
                config.push_str("    }\n");
                config.push_str("\n    location / {\n");
                config.push_str("        try_files $uri $uri/ =404;\n");
                config.push_str("    }\n");
            }
            config.push_str("}\n\n");
        }
    }

    config
}

/// 测试Nginx配置
pub async fn test_config(nginx_bin: &str) -> NginxTestResult {
    use tokio::process::Command;

    #[cfg(target_os = "linux")]
    {
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
            Ok(out) => NginxTestResult {
                success: out.status.success(),
                message: String::from_utf8_lossy(&out.stderr).to_string(),
            },
            Err(e) => NginxTestResult {
                success: false,
                message: format!("执行nginx命令失败: {}", e),
            },
        }
    }

    #[cfg(target_os = "windows")]
    {
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
            Ok(out) => NginxTestResult {
                success: out.status.success(),
                message: String::from_utf8_lossy(&out.stderr).to_string(),
            },
            Err(e) => NginxTestResult {
                success: false,
                message: format!("执行nginx命令失败: {}", e),
            },
        }
    }
}

/// 写入站点配置文件
pub async fn write_site_config(
    sites_enabled: &str,
    site_name: &str,
    config: &str,
) -> anyhow::Result<()> {
    let config_path = format!("{}/{}.conf", sites_enabled, site_name);
    tracing::info!("write_site_config: config_path={}", config_path);

    #[cfg(target_os = "linux")]
    {
        use tokio::process::Command;

        tokio::fs::create_dir_all(sites_enabled).await?;
        let tmp = "/tmp/.oxnginx_conf_tmp";
        tokio::fs::write(tmp, config).await?;
        let output = Command::new("mv")
            .args([tmp, &config_path])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("写入站点配置失败: {}", stderr));
        }
    }

    #[cfg(target_os = "windows")]
    {
        tokio::fs::create_dir_all(sites_enabled).await?;
        tokio::fs::write(&config_path, config).await?;
    }

    Ok(())
}

/// 删除站点配置文件
pub async fn remove_site_config(sites_enabled: &str, site_name: &str) -> anyhow::Result<()> {
    let config_path = format!("{}/{}.conf", sites_enabled, site_name);

    #[cfg(target_os = "linux")]
    {
        use tokio::process::Command;

        let _ = Command::new("rm")
            .args(["-f", &config_path])
            .output()
            .await;
    }

    #[cfg(target_os = "windows")]
    {
        if tokio::fs::metadata(&config_path).await.is_ok() {
            tokio::fs::remove_file(&config_path).await?;
        }
    }

    Ok(())
}

/// 确保 nginx.conf 中包含 sites-enabled 目录的 include 指令
pub async fn ensure_sites_enabled_include(nginx_config: &str, sites_enabled: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        use tokio::process::Command;

        // 读取原文件（root 用户可直接读取）
        let content = tokio::fs::read_to_string(nginx_config).await?;
        if content.contains("sites-enabled") {
            return Ok(());
        }
        let sites_path = sites_enabled.replace('\\', "/");
        let include_line = format!("\n    include {}/*.conf;\n", sites_path);
        if let Some(pos) = content.rfind('}') {
            let mut new_content = content[..pos].to_string();
            new_content.push_str(&include_line);
            new_content.push_str("}\n");
            let tmp = "/tmp/.oxnginx_nginx_conf_tmp";
            tokio::fs::write(tmp, &new_content).await?;
            let _ = Command::new("mv")
                .args([tmp, nginx_config])
                .output()
                .await;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let content = tokio::fs::read_to_string(nginx_config).await?;
        if content.contains("sites-enabled") {
            return Ok(());
        }
        let sites_path = sites_enabled.replace('\\', "/");
        let include_line = format!("\n    include {}/*.conf;\n", sites_path);
        if let Some(pos) = content.rfind('}') {
            let mut new_content = content[..pos].to_string();
            new_content.push_str(&include_line);
            new_content.push_str("}\n");
            tokio::fs::write(nginx_config, new_content).await?;
        }
        Ok(())
    }
}

/// 创建默认 index.html
pub async fn create_default_index(root_path: &str) -> anyhow::Result<()> {
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

    #[cfg(target_os = "linux")]
    {
        use tokio::process::Command;
        tokio::fs::create_dir_all(root_path).await?;
        let index_path = format!("{}/index.html", root_path);
        tokio::fs::write(&index_path, content).await?;
    }

    #[cfg(target_os = "windows")]
    {
        tokio::fs::create_dir_all(root_path).await?;
        tokio::fs::write(&format!("{}/index.html", root_path), content).await?;
    }

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

    // tracing::debug!("[NginxStatus] 检测 nginx: bin={}, os={}", nginx_bin, OS);

    // 先检测 nginx 是否可执行
    let version_check = Command::new(nginx_bin).arg("-v").output().await;
    let not_installed = version_check.is_err();

    // tracing::debug!("[NginxStatus] not_installed={}, version_check_err={}", not_installed, version_check.is_err());

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
pub async fn start_nginx(nginx_bin: &str, config_path: &str) -> anyhow::Result<bool> {
    use tokio::process::Command;

    // 先检查是否已在运行
    let status = get_nginx_status(nginx_bin).await;
    tracing::info!("启动前状态: running={}, pid={:?}, not_installed={}", status.running, status.pid, status.not_installed);
    if status.running {
        tracing::info!("Nginx 已在运行 (PID {:?})，跳过启动", status.pid);
        return Ok(true);
    }

    let nginx_dir = get_nginx_dir(nginx_bin)
        .ok_or_else(|| anyhow::anyhow!("无法获取 nginx 安装目录"))?;
    tracing::info!("启动 Nginx: bin={}, conf={}", nginx_bin, config_path);

    // spawn 不等待进程结束（nginx 在 Windows 上以前台模式运行）
    let child = Command::new(nginx_bin)
        .current_dir(&nginx_dir)
        .args(["-c", config_path])
        .spawn();

    match child {
        Ok(_) => {
            // 等一小段时间确认进程启动成功
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let status = get_nginx_status(nginx_bin).await;
            if status.running {
                tracing::info!("Nginx 启动成功 (PID {:?})", status.pid);
                Ok(true)
            } else {
                // nginx 可能启动后立即退出（配置错误等）
                Err(anyhow::anyhow!("Nginx 启动后未检测到进程，请检查配置"))
            }
        }
        Err(e) => Err(anyhow::anyhow!("启动 Nginx 失败: {}", e)),
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
pub async fn restart_nginx(nginx_bin: &str, config_path: &str) -> anyhow::Result<bool> {
    stop_nginx(nginx_bin).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    start_nginx(nginx_bin, config_path).await
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
        // Linux: 解压预编译 nginx 到 /opt/oxnginx/server/nginx/
        let src_tar = "/opt/oxnginx/server/nginx-src/nginx-1.30.3-linux-x86_64.tar.gz";

        // 检查预编译包是否存在
        if !std::path::Path::new(src_tar).exists() {
            return Err(anyhow::anyhow!(
                "nginx 预编译包不存在: {}，请确保部署时包含了 libs/nginx/linux/nginx-1.30.3-linux-x86_64.tar.gz",
                src_tar
            ));
        }

        // 解压到临时目录
        let tmp_dir = "/tmp/oxnginx-nginx-extract";
        let _ = tokio::fs::remove_dir_all(tmp_dir).await;
        tokio::fs::create_dir_all(tmp_dir).await?;

        tracing::info!("解压 nginx 预编译包...");
        let output = Command::new("tar")
            .args(["-xzf", src_tar, "-C", tmp_dir])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = tokio::fs::remove_dir_all(tmp_dir).await;
            return Err(anyhow::anyhow!("解压 nginx 失败: {}", stderr));
        }

        // 移动到目标目录
        let _ = tokio::fs::remove_dir_all(install_dir).await;
        tokio::fs::rename(format!("{}/nginx", tmp_dir), install_dir).await?;
        let _ = tokio::fs::remove_dir_all(tmp_dir).await;

        // 创建日志软链接
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::symlink;
            let logs_dir = format!("{}/logs", install_dir);
            let _ = symlink("/opt/oxnginx/wwwlogs", &logs_dir);
        }

        // 确保 sites-enabled 目录存在
        let sites_enabled = format!("{}/conf/sites-enabled", install_dir);
        tokio::fs::create_dir_all(&sites_enabled).await?;

        let bin = format!("{}/sbin/nginx", install_dir);
        let nginx_conf = format!("{}/conf/nginx.conf", install_dir);
        tracing::info!("Nginx 安装完成: {}", bin);
        Ok(NginxInstallResult {
            bin,
            config: nginx_conf,
            sites_enabled,
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
