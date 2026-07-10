use serde::Deserialize;

use crate::modules::site::entity::reverse_proxy::ReverseProxy;
use crate::modules::site::entity::site::Site;

/// 重定向规则（新版）
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RedirectRule {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub keep_params: bool,
    #[serde(default = "default_redirect_type")]
    pub redirect_type: String,
    #[serde(default = "default_redirect_method")]
    pub redirect_method: u16,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub target_url: String,
    #[serde(default = "default_status")]
    pub status: i32, // 1=启用 0=禁用
}

fn default_redirect_type() -> String { "type".to_string() }
fn default_redirect_method() -> u16 { 301 }
fn default_status() -> i32 { 1 }

/// 防盗链配置
#[derive(Debug, Deserialize)]
pub struct HotlinkConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default = "default_hotlink_code")]
    pub return_code: u16,
}

fn default_hotlink_code() -> u16 { 403 }

/// 解析 server_name 中的 host:port，返回 (纯 host 列表, 额外端口列表)
pub fn parse_server_names(server_name: &str) -> (Vec<String>, Vec<u16>) {
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
pub fn append_common_directives(config: &mut String, site: &Site, indent: &str) {
    if let Some(ref path) = site.log_access_path {
        config.push_str(&format!("{}access_log {};\n", indent, path));
    }
    if let Some(ref path) = site.log_error_path {
        config.push_str(&format!("{}error_log {};\n", indent, path));
    }

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

    if let Some(ref json) = site.redirect_rules {
        if let Ok(rules) = serde_json::from_str::<Vec<RedirectRule>>(json) {
            for rule in &rules {
                if rule.status != 1 || rule.target_url.is_empty() {
                    continue;
                }
                let code = rule.redirect_method;
                let target = if rule.keep_params {
                    format!("{}$args", rule.target_url)
                } else {
                    rule.target_url.clone()
                };
                for domain in &rule.domains {
                    if rule.redirect_type == "path" {
                        config.push_str(&format!("\n{}location {} {{\n", indent, domain));
                    } else {
                        let host = domain.split(':').next().unwrap_or(domain);
                        config.push_str(&format!("\n{}if ($host = {}) {{\n", indent, host));
                    }
                    config.push_str(&format!("{}    return {} {};\n", indent, code, target));
                    config.push_str(&format!("{}}}\n", indent));
                }
            }
        } else {
            #[derive(Debug, Deserialize)]
            struct LegacyRedirectRule {
                domain: String,
                target: String,
                #[serde(default)]
                redirect_type: u16,
            }
            if let Ok(rules) = serde_json::from_str::<Vec<LegacyRedirectRule>>(json) {
                for rule in &rules {
                    let code = if rule.redirect_type == 302 { 302 } else { 301 };
                    config.push_str(&format!("\n{}location {} {{\n", indent, rule.domain));
                    config.push_str(&format!("{}    return {} {};\n", indent, code, rule.target));
                    config.push_str(&format!("{}}}\n", indent));
                }
            }
        }
    }

    if let Some(ref rules) = site.rewrite_rules {
        let trimmed = rules.trim();
        if !trimmed.is_empty() {
            config.push('\n');
            for line in trimmed.lines() {
                config.push_str(&format!("{}{}\n", indent, line.trim_end()));
            }
        }
    }
}

/// 生成反向代理 location 块
pub fn append_proxy_locations(config: &mut String, proxies: &[ReverseProxy], indent: &str) {
    for proxy in proxies {
        if proxy.status != 1 {
            continue;
        }
        config.push_str(&format!("\n{}location {} {{\n", indent, proxy.proxy_dir));
        config.push_str(&format!("{}    proxy_pass {};\n", indent, proxy.target_url));
        config.push_str(&format!("{}    proxy_set_header Host $host;\n", indent));
        config.push_str(&format!("{}    proxy_set_header X-Real-IP $remote_addr;\n", indent));
        config.push_str(&format!("{}    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n", indent));
        config.push_str(&format!("{}    proxy_set_header X-Forwarded-Proto $scheme;\n", indent));
        config.push_str(&format!("{}    proxy_http_version 1.1;\n", indent));
        config.push_str(&format!("{}    proxy_set_header Upgrade $http_upgrade;\n", indent));
        config.push_str(&format!("{}    proxy_set_header Connection \"upgrade\";\n", indent));
        if proxy.cache == 1 {
            config.push_str(&format!("{}    proxy_cache_valid 200 302 10m;\n", indent));
        }
        config.push_str(&format!("{}}}\n", indent));
    }
}

