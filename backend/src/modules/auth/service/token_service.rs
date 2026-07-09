use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::auth::dao::token_dao;
use crate::modules::auth::entity::token::Token;

/// 创建 token（登录时调用）
pub async fn create_token(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    expires_hours: i64,
) -> Result<String> {
    Ok(token_dao::insert_token(pool, user_id, username, expires_hours).await?)
}

/// 验证 token（返回用户名，未找到或过期返回 None）
pub async fn verify_token(pool: &SqlitePool, token_str: &str) -> Result<Option<String>> {
    let now = chrono::Utc::now().naive_utc();
    match token_dao::find_token_username_expires(pool, token_str).await? {
        Some((username, expires_at)) if expires_at > now => Ok(Some(username)),
        _ => Ok(None),
    }
}

/// 验证 token（返回 Token 完整信息）
pub async fn verify_token_full(pool: &SqlitePool, token_str: &str) -> Result<Option<Token>> {
    let now = chrono::Utc::now().naive_utc();
    match token_dao::find_token_full(pool, token_str).await? {
        Some(t) if t.expires_at > now => Ok(Some(t)),
        _ => Ok(None),
    }
}

pub async fn delete_token(pool: &SqlitePool, token_str: &str) -> Result<()> {
    Ok(token_dao::delete_token(pool, token_str).await?)
}

pub async fn delete_user_tokens(pool: &SqlitePool, user_id: i64) -> Result<()> {
    Ok(token_dao::delete_user_tokens(pool, user_id).await?)
}

pub async fn refresh_token(pool: &SqlitePool, token_str: &str, expires_hours: i64) -> Result<()> {
    Ok(token_dao::refresh_token_expires(pool, token_str, expires_hours).await?)
}