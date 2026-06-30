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

    let output = Command::new(nginx_bin).arg("-t").output().await;

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

/// 重载Nginx配置
pub async fn reload_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    use tokio::process::Command;

    let output = Command::new(nginx_bin)
        .args(["-s", "reload"])
        .output()
        .await?;

    Ok(output.status.success())
}

/// 写入站点配置文件
pub async fn write_site_config(
    sites_enabled: &str,
    site_name: &str,
    config: &str,
) -> anyhow::Result<()> {
    let config_path = format!("{}/{}.conf", sites_enabled, site_name);
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

/// Nginx 进程状态
#[derive(Debug, serde::Serialize)]
pub struct NginxStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub version: Option<String>,
    pub uptime: Option<String>,
}

/// 获取 Nginx 运行状态
pub async fn get_nginx_status(nginx_bin: &str) -> NginxStatus {
    use tokio::process::Command;

    // 检查进程是否存在
    let pid_output = Command::new("pgrep")
        .args(["-x", "nginx"])
        .output()
        .await;

    let pid = match pid_output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().next().and_then(|p| p.trim().parse::<u32>().ok())
        }
        _ => None,
    };

    let running = pid.is_some();

    // 获取版本
    let version = if running {
        let version_output = Command::new(nginx_bin)
            .arg("-v")
            .output()
            .await;
        match version_output {
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                stderr.lines()
                    .find(|l| l.contains("version"))
                    .map(|l| l.trim().to_string())
            }
            _ => None,
        }
    } else {
        None
    };

    // 获取运行时间
    let uptime = if let Some(pid) = pid {
        let uptime_output = Command::new("ps")
            .args(["-o", "etime=", "-p", &pid.to_string()])
            .output()
            .await;
        match uptime_output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Some(stdout.trim().to_string())
            }
            _ => None,
        }
    } else {
        None
    };

    NginxStatus {
        running,
        pid,
        version,
        uptime,
    }
}

/// 启动 Nginx
pub async fn start_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    use tokio::process::Command;

    let output = Command::new(nginx_bin)
        .output()
        .await?;

    Ok(output.status.success())
}

/// 停止 Nginx
pub async fn stop_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    use tokio::process::Command;

    let output = Command::new(nginx_bin)
        .args(["-s", "stop"])
        .output()
        .await?;

    Ok(output.status.success())
}

/// 重启 Nginx
pub async fn restart_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    // 先停止
    stop_nginx(nginx_bin).await?;
    // 等待一小段时间
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    // 再启动
    start_nginx(nginx_bin).await
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
