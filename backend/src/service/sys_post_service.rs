use anyhow::Result;
use sqlx::SqlitePool;

use crate::model::Post;

// ============== 岗位 CRUD ==============

/// 分页查询岗位
pub async fn list_posts_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> Result<(Vec<Post>, i64)> {
    let offset = (page - 1).max(0) * page_size;
    let like = keyword.map(|k| format!("%{}%", k));
    let (total, rows) = if let Some(ref pattern) = like {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sys_posts WHERE name LIKE ? OR code LIKE ?",
        )
        .bind(pattern)
        .bind(pattern)
        .fetch_one(pool)
        .await?;
        let rows = sqlx::query_as::<_, Post>(
            "SELECT * FROM sys_posts WHERE name LIKE ? OR code LIKE ? ORDER BY sort, id LIMIT ? OFFSET ?",
        )
        .bind(pattern)
        .bind(pattern)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    } else {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sys_posts")
                .fetch_one(pool)
                .await?;
        let rows = sqlx::query_as::<_, Post>(
            "SELECT * FROM sys_posts ORDER BY sort, id LIMIT ? OFFSET ?",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    };
    Ok((rows, total))
}

/// 全量岗位列表
pub async fn list_posts(pool: &SqlitePool) -> Result<Vec<Post>> {
    Ok(sqlx::query_as::<_, Post>(
        "SELECT * FROM sys_posts WHERE status='enabled' ORDER BY sort, id",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_post(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    sort: i32,
) -> Result<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_posts (code, name, sort) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(code)
    .bind(name)
    .bind(sort)
    .fetch_one(pool)
    .await?)
}

pub async fn update_post(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    sort: Option<i32>,
) -> Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_posts SET name=? WHERE id=?")
            .bind(n).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_posts SET sort=? WHERE id=?")
            .bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_post(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_posts WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}
