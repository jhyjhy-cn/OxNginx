use axum::{extract::{ConnectInfo, Extension, State}, http::header, Json};
use serde_json::json;

use crate::dto::{ApiResponse, ChangePasswordRequest, ChangeUsernameRequest, LoginRequest, LoginResponse};
use crate::middleware::TokenInfo;
use crate::AppState;

/// 获取 RSA 公钥（公开接口）
pub async fn get_public_key(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!(ApiResponse::success(serde_json::json!({
        "public_key": state.rsa_public_key_b64
    }))))
}

/// 用户登录
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: header::HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Json<serde_json::Value> {
    // 提取 IP 和 UA
    let ip = crate::util::ua_parser::extract_ip(&headers, Some(addr));
    let ua = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let browser = crate::util::ua_parser::parse_browser(&ua);
    let os = crate::util::ua_parser::parse_os(&ua);

    // RSA 解密密码
    let password = match crate::auth::rsa_decrypt(&state.rsa_private_key, &req.encrypted_password) {
        Ok(p) => p,
        Err(_) => return Json(json!(ApiResponse::<()>::error("密码解密失败"))),
    };

    // 查询用户
    let user = sqlx::query_as::<_, crate::model::User>(
        "SELECT * FROM sys_users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(state.db.pool())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            let _ = crate::service::log_service::log_login(state.db.pool(), &req.username, Some(&ip), Some(&os), Some(&browser), Some(&ua), "login", "failed").await;
            return Json(json!(ApiResponse::<()>::error("用户名或密码错误")));
        }
        Err(e) => {
            tracing::error!("数据库错误: {}", e);
            return Json(json!(ApiResponse::<()>::error(format!("数据库错误: {}", e))));
        }
    };

    // 验证密码
    match crate::auth::verify_password(&password, &user.password) {
        Ok(true) => {}
        _ => {
            let _ = crate::service::log_service::log_login(state.db.pool(), &req.username, Some(&ip), Some(&os), Some(&browser), Some(&ua), "login", "failed").await;
            return Json(json!(ApiResponse::<()>::error("用户名或密码错误")));
        }
    };

    let config = state.get_config();
    let expires_hours = config.auth.token_expires_hours as i64;

    match crate::service::token_service::create_token(
        state.db.pool(),
        user.id,
        &user.username,
        expires_hours,
    )
    .await
    {
        Ok(token) => {
            let _ = crate::service::log_service::log_login(state.db.pool(), &user.username, Some(&ip), Some(&os), Some(&browser), Some(&ua), "login", "success").await;
            Json(json!(ApiResponse::success(LoginResponse {
                token,
                username: user.username,
            })))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("生成token失败: {}", e)))),
    }
}

/// 登出
pub async fn logout(State(state): State<AppState>, headers: header::HeaderMap) -> Json<serde_json::Value> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if let Some(token) = token {
        if let Ok(Some(username)) = crate::service::token_service::verify_token(state.db.pool(), token).await {
            let _ = crate::service::token_service::delete_token(state.db.pool(), token).await;
            let _ = crate::service::log_service::log_login(state.db.pool(), &username, None, None, None, None, "logout", "success").await;
        }
    }

    Json(json!(ApiResponse::success("ok")))
}

/// 修改密码（需要认证）
pub async fn change_password(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
    Json(req): Json<ChangePasswordRequest>,
) -> Json<serde_json::Value> {
    // 查询当前用户
    let user = sqlx::query_as::<_, crate::model::User>(
        "SELECT * FROM sys_users WHERE username = ?",
    )
    .bind(&info.username)
    .fetch_optional(state.db.pool())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("用户不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("数据库错误: {}", e)))),
    };

    // 验证旧密码
    match crate::auth::verify_password(&req.old_password, &user.password) {
        Ok(true) => {}
        _ => return Json(json!(ApiResponse::<()>::error("旧密码错误"))),
    }

    // 哈希新密码并更新
    let hashed = match crate::auth::hash_password(&req.new_password) {
        Ok(h) => h,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("密码哈希失败: {}", e)))),
    };

    match sqlx::query("UPDATE sys_users SET password = ? WHERE username = ?")
        .bind(&hashed)
        .bind(&info.username)
        .execute(state.db.pool())
        .await
    {
        Ok(_) => {
            // 使该用户所有旧 token 失效
            let _ = crate::service::token_service::delete_user_tokens(state.db.pool(), user.id).await;
            Json(json!(ApiResponse::success("密码修改成功")))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("修改密码失败: {}", e)))),
    }
}

/// 修改账号（需要认证）
pub async fn change_username(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
    Json(req): Json<ChangeUsernameRequest>,
) -> Json<serde_json::Value> {
    // 查询当前用户
    let user = sqlx::query_as::<_, crate::model::User>(
        "SELECT * FROM sys_users WHERE username = ?",
    )
    .bind(&info.username)
    .fetch_optional(state.db.pool())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return Json(json!(ApiResponse::<()>::error("用户不存在"))),
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("数据库错误: {}", e)))),
    };

    // 验证密码
    match crate::auth::verify_password(&req.password, &user.password) {
        Ok(true) => {}
        _ => return Json(json!(ApiResponse::<()>::error("密码错误"))),
    }

    // 检查新用户名是否已存在
    let exists: bool = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_users WHERE username = ?")
        .bind(&req.new_username)
        .fetch_one(state.db.pool())
        .await
        .map(|c| c > 0)
        .unwrap_or(false);

    if exists {
        return Json(json!(ApiResponse::<()>::error("用户名已存在")));
    }

    // 更新用户名
    match sqlx::query("UPDATE sys_users SET username = ? WHERE username = ?")
        .bind(&req.new_username)
        .bind(&info.username)
        .execute(state.db.pool())
        .await
    {
        Ok(_) => {
            // 更新 token 表中的用户名
            let _ = sqlx::query("UPDATE sys_tokens SET username = ? WHERE user_id = ?")
                .bind(&req.new_username)
                .bind(user.id)
                .execute(state.db.pool())
                .await;

            let (roles, permissions, menus) = match crate::service::rbac_service::get_rbac_info(
                state.db.pool(),
                &req.new_username,
            )
            .await
            {
                Ok(v) => v,
                Err(_) => (vec![], vec![], vec![]),
            };
            Json(json!(ApiResponse::success(serde_json::json!({
                "username": req.new_username,
                "roles": roles,
                "permissions": permissions,
                "menus": menus,
            }))))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("修改账号失败: {}", e)))),
    }
}

/// 检查是否需要初始化（公开接口，无需认证）
pub async fn setup_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_users")
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
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_users")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    if count > 0 {
        return Json(json!(ApiResponse::<()>::error("管理员账户已存在")));
    }

    // RSA 解密密码
    let password = match crate::auth::rsa_decrypt(&state.rsa_private_key, &req.encrypted_password) {
        Ok(p) => p,
        Err(_) => return Json(json!(ApiResponse::<()>::error("密码解密失败"))),
    };

    let hashed_password = match crate::auth::hash_password(&password) {
        Ok(h) => h,
        Err(e) => return Json(json!(ApiResponse::<()>::error(format!("密码哈希失败: {}", e)))),
    };

    let result = sqlx::query("INSERT INTO sys_users (username, password) VALUES (?, ?)")
        .bind(&req.username)
        .bind(&hashed_password)
        .execute(state.db.pool())
        .await;

    match result {
        Ok(_) => {
            let _ = crate::database::seed::run(state.db.pool()).await;
            Json(json!(ApiResponse::success("管理员账户创建成功")))
        }
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
        "SELECT * FROM sys_certificates WHERE domain = ?"
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
