use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use std::net::SocketAddr;

use crate::AppState;

/// 请求日志中间件
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();
    let response = next.run(request).await;
    tracing::info!("{} {} - {} ({:.1}ms)", method, uri, response.status(), start.elapsed().as_secs_f64() * 1000.0);
    response
}

/// Token 信息
#[derive(Clone, Debug)]
pub struct TokenInfo {
    pub username: String,
    #[allow(dead_code)]
    pub user_id: i64,
}

/// 客户端 IP 信息
#[derive(Clone, Debug)]
pub struct ClientIp(pub Option<String>);

/// Token 认证中间件
pub async fn auth_middleware(
    state: axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取客户端 IP
    let client_ip = get_client_ip(&request);
    request.extensions_mut().insert(ClientIp(client_ip.clone()));

    let auth_header = request.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    let token_str = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    match crate::service::token_service::verify_token_full(state.db.pool(), token_str).await {
        Ok(Some(token)) => {
            let expires_hours = state.get_config().auth.token_expires_hours as i64;
            let pool = state.db.pool().clone();
            let tk = token_str.to_string();
            tokio::spawn(async move { let _ = crate::service::token_service::refresh_token(&pool, &tk, expires_hours).await; });
            request.extensions_mut().insert(TokenInfo { username: token.username.clone(), user_id: token.user_id });
            tracing::debug!("[AUTH] Inserted TokenInfo: username={}, ip={:?}", token.username, client_ip);
            Ok(next.run(request).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// 获取客户端 IP
fn get_client_ip(request: &Request) -> Option<String> {
    // 优先从 x-forwarded-for 获取
    if let Some(ip) = request.headers().get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string()) {
        return Some(ip);
    }
    // 从 x-real-ip 获取
    if let Some(ip) = request.headers().get("x-real-ip")
        .and_then(|v| v.to_str().ok()) {
        return Some(ip.to_string());
    }
    // 从 ConnectInfo 获取（如果有的话）
    if let Some(conn_info) = request.extensions().get::<axum::extract::ConnectInfo<SocketAddr>>() {
        return Some(conn_info.0.ip().to_string());
    }
    None
}

/// 管理员认证中间件
/// ponytail: 自带 token 解析，不依赖上游 auth_middleware 注入 TokenInfo。
/// 即使 axum layer 顺序导致上游未跑，本中间件也能独立完成认证+授权。
pub async fn require_admin(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 优先用上游注入的 TokenInfo（避免重复查库）；缺失时自己解析 Authorization
    let username = if let Some(info) = request.extensions().get::<TokenInfo>().cloned() {
        info.username
    } else {
        let token = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        let Some(token) = token else { return Err(StatusCode::UNAUTHORIZED) };
        match crate::service::token_service::verify_token(state.db.pool(), token).await {
            Ok(Some(u)) => u,
            _ => return Err(StatusCode::UNAUTHORIZED),
        }
    };
    if username == "admin" { return Ok(next.run(request).await); }
    match crate::service::rbac_service::user_is_super_admin(&state.db.pool(), &username).await {
        Ok(true) => Ok(next.run(request).await),
        _ => Err(StatusCode::FORBIDDEN),
    }
}
