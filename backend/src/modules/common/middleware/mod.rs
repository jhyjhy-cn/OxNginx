use axum::{
    body::to_bytes,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Instant;
use std::net::SocketAddr;
use tokio::sync::Mutex;

use crate::AppState;

/// Token 续期去抖窗口（5 分钟）。
/// ponytail: 文件级 Mutex<HashMap<token, Instant>> 单点去抖;锁内不跨 await,try_lock 取不到直接放过。
const REFRESH_DEBOUNCE: std::time::Duration = std::time::Duration::from_secs(300);
static LAST_REFRESH: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();

fn last_refresh_map() -> &'static Mutex<HashMap<String, Instant>> {
    LAST_REFRESH.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 请求日志中间件:4xx/5xx 自动打 ERROR(含 body 摘要),正常 2xx/3xx 走 INFO
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();
    let response = next.run(request).await;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        // ponytail: 尝试读 body 拿错误详情(限 4KB,避免大 body 撑爆日志)
        let ct = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if ct.starts_with("application/json") || ct.starts_with("text/") {
            let (parts, body) = response.into_parts();
            let bytes = to_bytes(body, 4 * 1024).await.unwrap_or_default();
            let s = String::from_utf8_lossy(&bytes).chars().take(2000).collect::<String>();
            tracing::error!(target: "http", "{} {} -> {} ({:.1}ms) body={}", method, uri, status, elapsed_ms, s);
            Response::from_parts(parts, axum::body::Body::from(bytes))
        } else {
            tracing::error!(target: "http", "{} {} -> {} ({:.1}ms)", method, uri, status, elapsed_ms);
            response
        }
    } else {
        tracing::info!("{} {} - {} ({:.1}ms)", method, uri, status, elapsed_ms);
        response
    }
}

/// Token 信息
#[derive(Clone, Debug)]
pub struct TokenInfo {
    pub username: String,
    #[allow(dead_code)]
    pub user_id: i64,
    #[allow(dead_code)]
    pub token_id: i64,
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
        // ponytail: WebSocket 客户端无法设 header，query 兜底；只在 /api/ws 路径生效
        _ => match request.uri().path() {
            "/api/ws" => extract_token_from_query(request.uri().query()),
            _ => return Err(StatusCode::UNAUTHORIZED),
        },
    };
    match crate::modules::auth::service::token_service::verify_token_full(state.db.pool(), token_str).await {
        Ok(Some(token)) => {
            let expires_hours = state.get_config().auth.token_expires_hours as i64;
            // 续期去抖：5 分钟内同一 token 只更新一次 expires_at
            let now = Instant::now();
            let should_refresh = match last_refresh_map().try_lock() {
                Ok(mut g) => match g.get(token_str) {
                    Some(prev) if now.duration_since(*prev) < REFRESH_DEBOUNCE => false,
                    _ => {
                        g.insert(token_str.to_string(), now);
                        true
                    }
                },
                Err(_) => true, // 取不到锁让另一个请求去刷新
            };
            if should_refresh {
                let pool = state.db.pool().clone();
                let tk = token_str.to_string();
                tokio::spawn(async move {
                    if let Err(e) = crate::modules::auth::service::token_service::refresh_token(&pool, &tk, expires_hours).await {
                        tracing::warn!(error=%e, "refresh_token failed");
                    }
                });
            }
            request.extensions_mut().insert(TokenInfo { username: token.username.clone(), user_id: token.user_id, token_id: token.id });
            // tracing::debug!("[AUTH] Inserted TokenInfo: username={}, ip={:?}", token.username, client_ip);
            Ok(next.run(request).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// 从 query 字符串里抠 token=...（手切，不引入 url 依赖）
fn extract_token_from_query(q: Option<&str>) -> &str {
    let Some(q) = q else { return "" };
    for pair in q.split('&') {
        if let Some(v) = pair.strip_prefix("token=") {
            // ponytail: ws 客户端构造 URL 时不 url-encode token（hex 字符串无特殊字符），无需解码
            return v;
        }
    }
    ""
}
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

