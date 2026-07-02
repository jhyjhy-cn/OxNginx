use axum::{extract::State, Json};
use serde_json::json;

use crate::auth;
use crate::dto::{ApiResponse, LoginRequest, LoginResponse};
use crate::AppState;

/// 用户登录
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Json<serde_json::Value> {
    // 查询用户
    let user = sqlx::query_as::<_, crate::model::User>(
        "SELECT * FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(state.db.pool())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Json(json!(ApiResponse::<()>::error("用户名或密码错误")));
        }
        Err(e) => {
            return Json(json!(ApiResponse::<()>::error(format!("数据库错误: {}", e))));
        }
    };

    // 验证密码
    match auth::verify_password(&req.password, &user.password) {
        Ok(true) => {}
        _ => {
            return Json(json!(ApiResponse::<()>::error("用户名或密码错误")));
        }
    }

    // 生成JWT
    let config = state.get_config();
    match auth::generate_token(
        &user.username,
        &config.auth.jwt_secret,
        config.auth.jwt_expires_hours,
    ) {
        Ok(token) => Json(json!(ApiResponse::success(LoginResponse {
            token,
            username: user.username,
        }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("生成token失败: {}", e)))),
    }
}

/// 检查是否需要初始化（公开接口，无需认证）
pub async fn setup_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    Json(json!(ApiResponse::success(serde_json::json!({
        "need_setup": count == 0
    }))))
}

/// 初始化设置（创建管理员账户）
pub async fn setup(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Json<serde_json::Value> {
    // 检查是否已有用户
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    if count > 0 {
        return Json(json!(ApiResponse::<()>::error("管理员账户已存在")));
    }

    // 哈希密码
    let hashed_password = match auth::hash_password(&req.password) {
        Ok(h) => h,
        Err(e) => {
            return Json(json!(ApiResponse::<()>::error(format!("密码哈希失败: {}", e))));
        }
    };

    // 创建用户
    let result = sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
        .bind(&req.username)
        .bind(&hashed_password)
        .execute(state.db.pool())
        .await;

    match result {
        Ok(_) => Json(json!(ApiResponse::success("管理员账户创建成功"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建用户失败: {}", e)))),
    }
}

/// 获取证书列表
pub async fn list_certificates(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match crate::service::cert_service::get_all_certs(&state).await {
        Ok(certs) => Json(json!(ApiResponse::success(certs))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取证书列表失败: {}", e)))),
    }
}

/// 申请证书
pub async fn apply_certificate(
    State(state): State<AppState>,
    Json(req): Json<crate::dto::ApplyCertRequest>,
) -> Json<serde_json::Value> {
    match crate::service::cert_service::apply_cert(&state, &req.domain).await {
        Ok(cert) => Json(json!(ApiResponse::success(cert))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("申请证书失败: {}", e)))),
    }
}

/// 续期证书
pub async fn renew_certificate(
    State(state): State<AppState>,
    Json(req): Json<crate::dto::ApplyCertRequest>,
) -> Json<serde_json::Value> {
    // 根据域名查找证书
    let cert = sqlx::query_as::<_, crate::model::Certificate>(
        "SELECT * FROM certificates WHERE domain = ?"
    )
    .bind(&req.domain)
    .fetch_optional(state.db.pool())
    .await;

    match cert {
        Ok(Some(c)) => {
            match crate::service::cert_service::renew_cert(&state, c.id).await {
                Ok(true) => Json(json!(ApiResponse::success("证书续期成功"))),
                Ok(false) => Json(json!(ApiResponse::<()>::error("证书续期失败"))),
                Err(e) => Json(json!(ApiResponse::<()>::error(format!("证书续期失败: {}", e)))),
            }
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("证书不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("查询证书失败: {}", e)))),
    }
}
