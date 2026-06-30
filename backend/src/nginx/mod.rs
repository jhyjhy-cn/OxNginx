use crate::dto::NginxTestResult;
use crate::model::Site;

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
