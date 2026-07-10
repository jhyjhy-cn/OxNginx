use crate::modules::common::nginx::common_directives::{append_common_directives, append_proxy_locations, parse_server_names};
use crate::modules::site::entity::reverse_proxy::ReverseProxy;
use crate::modules::site::entity::site::Site;

/// 生成Nginx站点配置
pub fn generate_site_config(site: &Site) -> String {
    let mut config = String::new();

    if site.ssl == 1 {
        let (hosts, extra_ports) = parse_server_names(&site.server_name);
        let server_name_clean = hosts.join(" ");
        let main_port: u16 = site.listen.parse().unwrap_or(80);

        let mut redirect_ports = vec![main_port];
        for p in &extra_ports {
            if *p != 443 && !redirect_ports.contains(p) {
                redirect_ports.push(*p);
            }
        }

        for port in &redirect_ports {
            config.push_str("server {\n");
            config.push_str(&format!("    listen {};\n", port));
            config.push_str(&format!("    listen [::]:{};\n", port));
            config.push_str(&format!("    server_name {};\n", server_name_clean));
            config.push_str("    return 301 https://$host$request_uri;\n");
            config.push_str("}\n\n");
        }

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
        let (hosts, extra_ports) = parse_server_names(&site.server_name);
        let server_name_clean = hosts.join(" ");
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

/// 生成带反向代理列表的站点配置
pub fn generate_site_config_with_proxies(site: &Site, proxies: &[ReverseProxy]) -> String {
    if proxies.is_empty() {
        return generate_site_config(site);
    }

    let mut config = String::new();

    if site.ssl == 1 {
        let (hosts, extra_ports) = parse_server_names(&site.server_name);
        let server_name_clean = hosts.join(" ");
        let main_port: u16 = site.listen.parse().unwrap_or(80);
        let mut redirect_ports = vec![main_port];
        for p in &extra_ports {
            if *p != 443 && !redirect_ports.contains(p) {
                redirect_ports.push(*p);
            }
        }

        for port in &redirect_ports {
            config.push_str("server {\n");
            config.push_str(&format!("    listen {};\n", port));
            config.push_str(&format!("    listen [::]:{};\n", port));
            config.push_str(&format!("    server_name {};\n", server_name_clean));
            config.push_str("    return 301 https://$host$request_uri;\n");
            config.push_str("}\n\n");
        }

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
            append_proxy_locations(&mut config, proxies, "    ");

            if let Some(root_path) = &site.root_path {
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
        let (hosts, extra_ports) = parse_server_names(&site.server_name);
        let server_name_clean = hosts.join(" ");
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
            append_proxy_locations(&mut config, proxies, "    ");

            if let Some(root_path) = &site.root_path {
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
        use crate::modules::common::util::cmd::silent_tokio_command;

        tokio::fs::create_dir_all(sites_enabled).await?;
        let tmp = "/tmp/.oxnginx_conf_tmp";
        tokio::fs::write(tmp, config).await?;
        let output = silent_tokio_command("mv")
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
        use crate::modules::common::util::cmd::silent_tokio_command;

        let _ = silent_tokio_command("rm")
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
