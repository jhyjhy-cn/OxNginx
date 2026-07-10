use sqlx::SqlitePool;

use crate::modules::sys::entity::param::Param;

/// 分页查询
pub async fn page_params(
    pool: &SqlitePool,
    keyword: Option<&str>,
    group_code: Option<&str>,
    page: i64,
    page_size: i64,
) -> sqlx::Result<(Vec<Param>, i64)> {
    let offset = (page - 1) * page_size;

    let like_kw = keyword.map(|k| format!("%{}%", k));
    let like_gc = group_code.map(|g| format!("%{}%", g));

    let rows = sqlx::query_as::<_, Param>(
        "SELECT * FROM sys_params
         WHERE (? IS NULL OR key LIKE ? OR name LIKE ?)
           AND (? IS NULL OR group_code LIKE ?)
         ORDER BY sort, id
         LIMIT ? OFFSET ?",
    )
    .bind(like_kw.as_ref())
    .bind(like_kw.as_ref())
    .bind(like_kw.as_ref())
    .bind(like_gc.as_ref())
    .bind(like_gc.as_ref())
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sys_params
         WHERE (? IS NULL OR key LIKE ? OR name LIKE ?)
           AND (? IS NULL OR group_code LIKE ?)",
    )
    .bind(like_kw.as_ref())
    .bind(like_kw.as_ref())
    .bind(like_kw.as_ref())
    .bind(like_gc.as_ref())
    .bind(like_gc.as_ref())
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn find_param_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Param>> {
    sqlx::query_as::<_, Param>("SELECT * FROM sys_params WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_param_by_key(pool: &SqlitePool, key: &str) -> sqlx::Result<Option<Param>> {
    sqlx::query_as::<_, Param>("SELECT * FROM sys_params WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
}

pub async fn insert_param_returning_id(
    pool: &SqlitePool,
    key: &str,
    value: Option<&str>,
    name: &str,
    group_code: &str,
    remark: Option<&str>,
    sort: i32,
) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_params (key, value, name, group_code, remark, sort)
         VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(key)
    .bind(value)
    .bind(name)
    .bind(group_code)
    .bind(remark)
    .bind(sort)
    .fetch_one(pool)
    .await
}

/// 局部更新：None 字段不改动
pub async fn update_param_fields(
    pool: &SqlitePool,
    id: i64,
    value: Option<&str>,
    name: Option<&str>,
    group_code: Option<&str>,
    remark: Option<&str>,
    sort: Option<i32>,
) -> sqlx::Result<()> {
    if let Some(v) = value {
        sqlx::query("UPDATE sys_params SET value = ? WHERE id = ?")
            .bind(v)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(n) = name {
        sqlx::query("UPDATE sys_params SET name = ? WHERE id = ?")
            .bind(n)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(g) = group_code {
        sqlx::query("UPDATE sys_params SET group_code = ? WHERE id = ?")
            .bind(g)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(r) = remark {
        sqlx::query("UPDATE sys_params SET remark = ? WHERE id = ?")
            .bind(r)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_params SET sort = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(s)
            .bind(id)
            .execute(pool)
            .await?;
    } else {
        sqlx::query("UPDATE sys_params SET updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn delete_param(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_params WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}