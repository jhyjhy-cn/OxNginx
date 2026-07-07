use anyhow::Result;
use chrono::{Duration, NaiveDateTime};
use sqlx::SqlitePool;

use crate::model::Token;

/// 生成随机 token
fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// 创建 token（登录时调用）
pub async fn create_token(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    expires_hours: i64,
) -> Result<String> {
    let token_str = generate_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(Duration::hours(expires_hours))
        .unwrap()
        .naive_utc();

    sqlx::query(
        "INSERT INTO sys_tokens (token, user_id, username, expires_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&token_str)
    .bind(user_id)
    .bind(username)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(token_str)
}

/// 验证 token（返回用户名，未找到或过期返回 None）
pub async fn verify_token(pool: &SqlitePool, token_str: &str) -> Result<Option<String>> {
    let now = chrono::Utc::now().naive_utc();

    let row: Option<(String, NaiveDateTime)> = sqlx::query_as(
        "SELECT username, expires_at FROM sys_tokens WHERE token = ?",
    )
    .bind(token_str)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((username, expires_at)) if expires_at > now => Ok(Some(username)),
        _ => Ok(None),
    }
}

/// 验证 token（返回 Token 完整信息）
pub async fn verify_token_full(pool: &SqlitePool, token_str: &str) -> Result<Option<Token>> {
    let now = chrono::Utc::now().naive_utc();

    let token: Option<Token> = sqlx::query_as(
        "SELECT id, token, user_id, username, expires_at, created_at FROM sys_tokens WHERE token = ?",
    )
    .bind(token_str)
    .fetch_optional(pool)
    .await?;

    match token {
        Some(t) if t.expires_at > now => Ok(Some(t)),
        _ => Ok(None),
    }
}

/// 删除 token（登出时调用）
pub async fn delete_token(pool: &SqlitePool, token_str: &str) -> Result<()> {
    sqlx::query("DELETE FROM sys_tokens WHERE token = ?")
        .bind(token_str)
        .execute(pool)
        .await?;
    Ok(())
}

/// 删除用户的所有 token（改密码时调用，使旧 token 全部失效）
pub async fn delete_user_tokens(pool: &SqlitePool, user_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM sys_tokens WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 清理过期 token（可选，定期调用）
pub async fn cleanup_expired_tokens(pool: &SqlitePool) -> Result<u64> {
    let now = chrono::Utc::now().naive_utc();
    let result = sqlx::query("DELETE FROM sys_tokens WHERE expires_at <= ?")
        .bind(now)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
