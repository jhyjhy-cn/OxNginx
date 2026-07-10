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
use crate::modules::common::audit::context::{AuditContext, SharedAuditContext};
use crate::modules::common::audit::event::AuditEvent;
use crate::modules::common::audit::sender;
use crate::modules::common::middleware::ClientIp;
use crate::modules::common::middleware::TokenInfo;

/// 操作日志中间件。
/// - 只记录 POST/PUT/DELETE
/// - 白名单内的 URI（list/search/...）放行
/// - multipart / octet-stream / >8MB body 跳过采集
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

    // 从 auth_middleware 注入的 ClientIp 获取
    let ip_from_auth = request
        .extensions()
        .get::<ClientIp>()
        .map(|c| c.0.clone())
        .unwrap_or_else(|| {
            tracing::warn!("[AUDIT] ClientIp NOT found");
            None
        });

    // 读取 body 并重建请求
    let skip_req = should_skip_body(request.headers());
    let (request, req_body_log) = if skip_req {
        (request, None)
    } else {
        let (parts, body) = request.into_parts();
        match body::to_bytes(body, 2 * 1024 * 1024).await {
            Ok(bytes) => {
                let log_str = truncate_str(&bytes);
                (Request::from_parts(parts, Body::from(bytes)), log_str)
            }
            Err(_) => {
                (Request::from_parts(parts, Body::empty()), None)
            }
        }
    };

    // 从 TokenInfo 获取 username
    let username = request
        .extensions()
        .get::<TokenInfo>()
        .map(|t| {
            // tracing::debug!("[AUDIT] TokenInfo found: username={}", t.username);
            t.username.clone()
        })
        .unwrap_or_else(|| {
            // tracing::warn!("[AUDIT] TokenInfo NOT found in request");
            "unknown".into()
        });

    let start = Instant::now();
    let response = next.run(request).await;
    let duration_ms = start.elapsed().as_millis() as i64;

    let ctx_data = ctx.lock().unwrap().clone();

    let status_code = response.status().as_u16();

    // 读取响应体并重建响应（需要在判断 status 之前读取）
    let (response, resp_body_log) = collect_response_body(response).await;

    // 同时检查 HTTP status 和响应体中的 code 字段
    let is_success_http = (200..300).contains(&status_code);
    let is_success_body = check_response_success(&resp_body_log);
    let status_str: i32 = if is_success_http && is_success_body { 1 } else { 0 };

    let error_msg = if status_str == 0 {
        ctx_data
            .error
            .clone()
            .or_else(|| Some(resp_body_log.clone().unwrap_or_else(|| format!("HTTP {}", status_code))))
    } else {
        ctx_data.error.clone()
    };

    let mut ev = AuditEvent::now(trace_id);
    ev.username = username;
    ev.module = ctx_data.module.unwrap_or_default();
    ev.action = ctx_data.action.unwrap_or_default();
    ev.method = method.to_string();
    ev.uri = uri_s;
    ev.ip = ip_from_auth;
    ev.status = status_str;
    ev.duration_ms = duration_ms;
    ev.request_body = req_body_log;
    ev.response_body = resp_body_log;
    ev.error_msg = error_msg;
    sender::submit(ev).await;

    response
}

/// 检查响应体中的 code 字段，ApiResponse::success 返回 code=0
fn check_response_success(resp_body: &Option<String>) -> bool {
    let Some(body) = resp_body else { return true };
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(code) = json.get("code").and_then(|v| v.as_i64()) {
            return code == 0;
        }
    }
    true
}

/// 读取响应体并重建响应
async fn collect_response_body(response: Response) -> (Response, Option<String>) {
    let (parts, body) = response.into_parts();

    // 使用 body::to_bytes 收集 body
    match body::to_bytes(body, 2 * 1024 * 1024).await {
        Ok(bytes) => {
            let resp_body_log = truncate_str(&bytes);
            let new_body = Body::from(bytes);
            (Response::from_parts(parts, new_body), resp_body_log)
        }
        Err(_) => {
            // 无法读取 body
            (Response::from_parts(parts, Body::empty()), None)
        }
    }
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
