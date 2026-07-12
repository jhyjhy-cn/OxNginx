use sqlx::SqlitePool;

/// 在线会话列表项（含用户名 + 部门名）
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct OnlineItem {
    pub id: i64,
    pub token: String,
    pub user_id: i64,
    pub username: String,
    pub nickname: Option<String>,
    pub dept_id: Option<i64>,
    pub dept_name: Option<String>,
    pub ip: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub expires_at: chrono::NaiveDateTime,
}

/// 分页 + 模糊查询（username / ip）
pub async fn list_online_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> sqlx::Result<(Vec<OnlineItem>, i64)> {
    let offset = (page - 1) * page_size;
    let like = keyword.map(|k| format!("%{}%", k));

    let list = sqlx::query_as::<_, OnlineItem>(
        r#"SELECT t.id, t.token, t.user_id, t.username,
                  u.nickname, u.dept_id, d.name AS dept_name,
                  t.ip, t.os, t.browser, t.user_agent,
                  t.created_at, t.expires_at
           FROM sys_tokens t
           LEFT JOIN sys_users u ON u.id = t.user_id
           LEFT JOIN sys_depts d ON d.id = u.dept_id
           WHERE t.expires_at > datetime('now')
             AND (?1 IS NULL OR t.username LIKE ?1 OR t.ip LIKE ?1)
           ORDER BY t.created_at DESC
           LIMIT ?2 OFFSET ?3"#,
    )
    .bind(&like)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM sys_tokens t
           WHERE t.expires_at > datetime('now')
             AND (?1 IS NULL OR t.username LIKE ?1 OR t.ip LIKE ?1)"#,
    )
    .bind(&like)
    .fetch_one(pool)
    .await?;

    Ok((list, total))
}