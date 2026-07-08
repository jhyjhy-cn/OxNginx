use axum::{
    body::{self, Body, Bytes},
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;

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

/// Token 认证中间件
pub async fn auth_middleware(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
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
            let mut request = request;
            request.extensions_mut().insert(TokenInfo { username: token.username, user_id: token.user_id });
            Ok(next.run(request).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// 管理员认证中间件
pub async fn require_admin(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token_info = request.extensions().get::<TokenInfo>().cloned();
    let Some(info) = token_info else { return Err(StatusCode::UNAUTHORIZED) };
    if info.username == "admin" { return Ok(next.run(request).await); }
    match crate::service::rbac_service::user_is_super_admin(&state.db.pool(), &info.username).await {
        Ok(true) => Ok(next.run(request).await),
        _ => Err(StatusCode::FORBIDDEN),
    }
}

/// 操作日志中间件 — 自动记录所有写操作（POST/PUT/DELETE）
pub async fn operation_log_middleware(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().to_string();
    let is_write = method == axum::http::Method::POST
        || method == axum::http::Method::PUT
        || method == axum::http::Method::DELETE;
    // 排除查询类 POST
    let is_query = uri.contains("/list") || uri.contains("/search") || uri.contains("/files/list")
        || uri.contains("/files/read") || uri.contains("/files/size") || uri.contains("/files/roots")
        || uri.contains("/log/") || uri.contains("/rbac/me") || uri.contains("/i18n/messages")
        || uri.contains("/preview") || uri.contains("/diff") || uri.contains("/setup/status")
        || uri.contains("/auth/public-key") || uri.contains("/dashboard");

    if !is_write || is_query {
        return next.run(request).await;
    }

    let username = request.extensions().get::<TokenInfo>().map(|t| t.username.clone());
    let ip = request.headers().get("x-forwarded-for").and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next()).map(|s| s.trim().to_string());

    // 读请求 body 并重建
    let (parts, req_body) = request.into_parts();
    let req_body_bytes = body::to_bytes(req_body, 2 * 1024 * 1024).await.unwrap_or_default();
    let req_body_log = truncate_body(&req_body_bytes);
    let request = Request::from_parts(parts, Body::from(req_body_bytes));

    let start = Instant::now();
    let response = next.run(request).await;
    let cost_ms = start.elapsed().as_millis() as i64;

    let ok = response.status().is_success();
    let (parts, res_body) = response.into_parts();
    let res_body_bytes = body::to_bytes(res_body, 2 * 1024 * 1024).await.unwrap_or_default();
    let res_body_log = truncate_body(&res_body_bytes);

    let action = format!("{} {}", method, uri);
    let u = username.unwrap_or_else(|| "unknown".into());
    let status = if ok { "success" } else { "failed" };
    let err = if !ok { Some(format!("HTTP {}", parts.status.as_u16())) } else { None };
    let pool = state.db.pool().clone();
    let m = method.to_string();
    tokio::spawn(async move {
        let _ = crate::service::log_service::log_operation(
            &pool, &u, &action, Some(&m), Some(&uri), ip.as_deref(),
            status, Some(cost_ms), req_body_log.as_deref(), res_body_log.as_deref(), err.as_deref(),
        ).await;
    });

    Response::from_parts(parts, Body::from(res_body_bytes))
}

fn truncate_body(bytes: &Bytes) -> Option<String> {
    if bytes.is_empty() { return None; }
    let s = String::from_utf8_lossy(bytes);
    Some(if s.len() > 2000 { format!("{}...", &s[..2000]) } else { s.to_string() })
}
