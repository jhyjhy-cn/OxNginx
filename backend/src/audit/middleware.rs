use std::time::Instant;

use axum::{
    body::{self, Body, Bytes},
    extract::{Request, State},
    http::{HeaderMap, Method},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::app::state::AppState;
use crate::audit::context::{AuditContext, SharedAuditContext};
use crate::audit::event::AuditEvent;
use crate::audit::sender;
use crate::middleware::TokenInfo;

/// 操作日志中间件。
/// - 只记录 POST/PUT/DELETE
/// - 白名单内的 URI（list/search/...）放行
/// - multipart / octet-stream / >8MB body 跳过采集
/// - 不读 response body（review #9：审计只关心 status / duration）
pub async fn audit_middleware(
    State(_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let is_write = matches!(method, Method::POST | Method::PUT | Method::DELETE);
    if !is_write {
        return next.run(request).await;
    }

    let uri_s = request.uri().path().to_string();
    if is_query_uri(&uri_s) {
        return next.run(request).await;
    }

    let trace_id = Uuid::new_v4().to_string();

    let ctx: SharedAuditContext = std::sync::Arc::new(std::sync::Mutex::new(AuditContext::default()));
    request.extensions_mut().insert(ctx.clone());

    // 读 body（按跳过策略）后用 from_parts 重建
    let (parts, body) = request.into_parts();
    let skip_req = should_skip_body(&parts.headers);
    let (request, req_body_log) = if skip_req {
        (Request::from_parts(parts, body), None)
    } else {
        match body::to_bytes(body, 2 * 1024 * 1024).await {
            Ok(bytes) => {
                let log_str = truncate_str(&bytes);
                (Request::from_parts(parts, Body::from(bytes)), log_str)
            }
            Err(_) => {
                // 兜底：to_bytes 失败时给空 body，handler 解析会失败但不影响日志其他字段
                (Request::from_parts(parts, Body::empty()), None)
            }
        }
    };

    let start = Instant::now();
    let response = next.run(request).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    // ponytail: 必须在 next.run() 返回后读 username / ip —— 此时 auth 中间件已把 TokenInfo 塞进 extensions
    let username = response
        .extensions()
        .get::<TokenInfo>()
        .map(|t| t.username.clone())
        .unwrap_or_else(|| "unknown".into());
    let ip_from_header = response
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string());

    let ctx_data = ctx.lock().unwrap().clone();

    let status_code = response.status().as_u16();
    let status_str = if (200..300).contains(&status_code) {
        "success"
    } else {
        "failed"
    };

    let error_msg = if status_str == "failed" {
        ctx_data
            .error
            .clone()
            .or_else(|| Some(format!("HTTP {}", status_code)))
    } else {
        ctx_data.error.clone()
    };

    let mut ev = AuditEvent::now(trace_id);
    ev.username = username;
    ev.module = ctx_data.module.unwrap_or_default();
    ev.action = ctx_data.action.unwrap_or_default();
    ev.method = method.to_string();
    ev.uri = uri_s;
    ev.ip = ip_from_header;
    ev.status = status_str.into();
    ev.duration_ms = duration_ms;
    ev.request_body = req_body_log;
    ev.error_msg = error_msg;
    sender::submit(ev).await;

    response
}

fn should_skip_body(headers: &HeaderMap) -> bool {
    if let Some(ct) = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
    {
        if ct.starts_with("multipart/") || ct.starts_with("application/octet-stream") {
            return true;
        }
    }
    if let Some(len) = headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
    {
        if len > 8 * 1024 * 1024 {
            return true;
        }
    }
    false
}

fn truncate_str(b: &Bytes) -> Option<String> {
    if b.is_empty() {
        return None;
    }
    let s = String::from_utf8_lossy(b);
    Some(if s.len() > 2000 {
        format!("{}...", &s[..2000])
    } else {
        s.to_string()
    })
}

fn is_query_uri(uri: &str) -> bool {
    uri.contains("/list")
        || uri.contains("/search")
        || uri.contains("/files/list")
        || uri.contains("/files/read")
        || uri.contains("/files/size")
        || uri.contains("/files/roots")
        || uri.contains("/log/")
        || uri.contains("/rbac/me")
        || uri.contains("/i18n/messages")
        || uri.contains("/preview")
        || uri.contains("/diff")
        || uri.contains("/setup/status")
        || uri.contains("/auth/public-key")
        || uri.contains("/dashboard")
}
