use sqlx::SqlitePool;

use crate::modules::auth::entity::token::Token;

/// 生成随机 token
fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// 创建 token（登录时调用）
pub async fn insert_token(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    expires_hours: i64,
) -> sqlx::Result<String> {
    let token_str = generate_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(expires_hours))
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

/// 验证 token，返回 (username, expires_at)
pub async fn find_token_username_expires(
    pool: &SqlitePool,
    token_str: &str,
) -> sqlx::Result<Option<(String, chrono::NaiveDateTime)>> {
    sqlx::query_as(
        "SELECT username, expires_at FROM sys_tokens WHERE token = ?",
    )
    .bind(token_str)
    .fetch_optional(pool)
    .await
}

/// 验证 token 完整信息
pub async fn find_token_full(pool: &SqlitePool, token_str: &str) -> sqlx::Result<Option<Token>> {
    sqlx::query_as(
        "SELECT id, token, user_id, username, expires_at, created_at FROM sys_tokens WHERE token = ?",
    )
    .bind(token_str)
    .fetch_optional(pool)
    .await
}

pub async fn delete_token(pool: &SqlitePool, token_str: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sys_tokens WHERE token = ?")
        .bind(token_str)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_user_tokens(pool: &SqlitePool, user_id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sys_tokens WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 滑动续期
pub async fn refresh_token_expires(
    pool: &SqlitePool,
    token_str: &str,
    expires_hours: i64,
) -> sqlx::Result<()> {
    let new_expires = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(expires_hours))
        .unwrap()
        .naive_utc();
    sqlx::query("UPDATE sys_tokens SET expires_at = ? WHERE token = ?")
        .bind(new_expires)
        .bind(token_str)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_expired_tokens(pool: &SqlitePool) -> sqlx::Result<u64> {
    let now = chrono::Utc::now().naive_utc();
    let r = sqlx::query("DELETE FROM sys_tokens WHERE expires_at <= ?")
        .bind(now)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}