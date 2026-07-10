use crate::modules::common::dto::CreateUpstreamRequest;
use crate::modules::site::entity::upstream::{Upstream, UpstreamServer};

/// 生成上游服务器配置
pub fn generate_upstream_config(upstream: &Upstream, servers: &[UpstreamServer]) -> String {
    let mut config = String::new();

    config.push_str(&format!("upstream {} {{\n", upstream.name));

    match upstream.method.as_str() {
        "ip_hash" => config.push_str("    ip_hash;\n"),
        "least_conn" => config.push_str("    least_conn;\n"),
        "hash" => config.push_str(&format!("    hash $request_uri;\n")),
        _ => {}
    }

    if upstream.keepalive > 0 {
        config.push_str(&format!("    keepalive {};\n", upstream.keepalive));
    }

    for server in servers {
        if server.status != 1 {
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

    match req.method.as_str() {
        "ip_hash" => config.push_str("    ip_hash;\n"),
        "least_conn" => config.push_str("    least_conn;\n"),
        "hash" => config.push_str(&format!("    hash $request_uri;\n")),
        _ => {}
    }

    if req.keepalive > 0 {
        config.push_str(&format!("    keepalive {};\n", req.keepalive));
    }

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
