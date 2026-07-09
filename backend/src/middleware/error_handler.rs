use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::dto::ApiResponse;

/// 应用全局异常类型
#[derive(Debug, Clone)]
pub struct AppError {
    pub code: i32,
    pub message: String,
}

impl AppError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, message)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("全局异常: {:?}", self);
        let status = match self.code {
            400 => StatusCode::BAD_REQUEST,
            404 => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!(ApiResponse::<()>::error(self.message)));
        (status, body).into_response()
    }
}

/// 从 anyhow::Error 转换为 AppError
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::internal(e.to_string())
    }
}

/// 从 String 转换为 AppError
impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::internal(s)
    }
}

/// 从 &str 转换为 AppError
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::internal(s.to_string())
    }
}

/// 全局异常捕获中间件
/// 捕获所有 handler 返回的 Err，并统一转换为 JSON 响应
pub async fn error_handler(request: Request, next: Next) -> Result<Response, AppError> {
    Ok(next.run(request).await)
}
