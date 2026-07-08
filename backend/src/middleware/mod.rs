use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::AppState;

/// 请求日志中间件
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let elapsed = start.elapsed();
    tracing::info!(
        "{} {} - {} ({:.1}ms)",
        method,
        uri,
        response.status(),
        elapsed.as_secs_f64() * 1000.0,
    );

    response
}

/// Token 信息（从中间件注入到请求上下文）
#[derive(Clone, Debug)]
pub struct TokenInfo {
    pub username: String,
    #[allow(dead_code)]
    pub user_id: i64,
}

/// Token 认证中间件（查数据库验证 token）
pub async fn auth_middleware(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token_str = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // 查数据库验证 token
    match crate::service::token_service::verify_token_full(state.db.pool(), token_str).await {
        Ok(Some(token)) => {
            // 滑动续期（fire-and-forget，不阻塞请求）
            let expires_hours = state.get_config().auth.token_expires_hours as i64;
            let pool = state.db.pool().clone();
            let tk = token_str.to_string();
            tokio::spawn(async move {
                let _ = crate::service::token_service::refresh_token(&pool, &tk, expires_hours).await;
            });
            let mut request = request;
            request.extensions_mut().insert(TokenInfo {
                username: token.username,
                user_id: token.user_id,
            });
            Ok(next.run(request).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// 管理员认证中间件（需 auth_middleware 之后使用）
pub async fn require_admin(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token_info = request.extensions().get::<TokenInfo>().cloned();
    let Some(info) = token_info else { return Err(StatusCode::UNAUTHORIZED) };

    // admin 用户直接通过
    if info.username == "admin" {
        return Ok(next.run(request).await);
    }

    // 其他用户查数据库验证 super_admin 角色
    match crate::service::rbac_service::user_is_super_admin(&state.db.pool(), &info.username).await {
        Ok(true) => Ok(next.run(request).await),
        _ => Err(StatusCode::FORBIDDEN),
    }
}
