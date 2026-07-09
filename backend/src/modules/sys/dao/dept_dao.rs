use sqlx::SqlitePool;

use crate::modules::sys::entity::dept::Dept;

pub async fn list_depts_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> sqlx::Result<(Vec<Dept>, i64)> {
    let offset = (page - 1).max(0) * page_size;
    let like = keyword.map(|k| format!("%{}%", k));
    if let Some(ref pattern) = like {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sys_depts WHERE name LIKE ?",
        )
        .bind(pattern)
        .fetch_one(pool)
        .await?;
        let rows = sqlx::query_as::<_, Dept>(
            "SELECT * FROM sys_depts WHERE name LIKE ? ORDER BY sort, id LIMIT ? OFFSET ?",
        )
        .bind(pattern)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok((rows, total))
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_depts")
            .fetch_one(pool)
            .await?;
        let rows = sqlx::query_as::<_, Dept>(
            "SELECT * FROM sys_depts ORDER BY sort, id LIMIT ? OFFSET ?",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok((rows, total))
    }
}

pub async fn list_depts(pool: &SqlitePool) -> sqlx::Result<Vec<Dept>> {
    sqlx::query_as::<_, Dept>("SELECT * FROM sys_depts ORDER BY sort, id")
        .fetch_all(pool)
        .await
}

pub async fn insert_dept_returning_id(
    pool: &SqlitePool,
    name: &str,
    parent_id: Option<i64>,
    sort: i32,
) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_depts (name, parent_id, sort) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(parent_id)
    .bind(sort)
    .fetch_one(pool)
    .await
}

pub async fn update_dept_fields(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    parent_id: Option<Option<i64>>,
    sort: Option<i32>,
) -> sqlx::Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_depts SET name=? WHERE id=?")
            .bind(n).bind(id).execute(pool).await?;
    }
    if let Some(p) = parent_id {
        sqlx::query("UPDATE sys_depts SET parent_id=? WHERE id=?")
            .bind(p).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_depts SET sort=? WHERE id=?")
            .bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_dept(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_depts WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}