use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::auth;
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

/// JWT认证中间件
pub async fn auth_middleware(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let config = state.get_config();
    match auth::verify_token(token, &config.auth.jwt_secret) {
        Ok(claims) => {
            let mut request = request;
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// ponytail: username=='admin' 短路；非 admin 查 DB 验证 super_admin 角色
/// 需挂在本中间件 *之后*，因为依赖 auth_middleware 注入的 Claims
pub async fn require_admin(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request.extensions().get::<auth::Claims>().cloned();
    let Some(c) = claims else { return Err(StatusCode::UNAUTHORIZED) };
    if c.sub == "admin" {
        return Ok(next.run(request).await);
    }
    match crate::service::rbac_service::user_is_super_admin(&state.db.pool(), &c.sub).await {
        Ok(true) => Ok(next.run(request).await),
        _ => Err(StatusCode::FORBIDDEN),
    }
}
