use sqlx::SqlitePool;

use crate::modules::sys::entity::dict::{Dict, DictItem};

pub async fn list_dicts(pool: &SqlitePool) -> sqlx::Result<Vec<Dict>> {
    sqlx::query_as::<_, Dict>("SELECT * FROM sys_dict ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn find_dict_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Dict>> {
    sqlx::query_as::<_, Dict>("SELECT * FROM sys_dict WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_dict_returning_id(
    pool: &SqlitePool,
    name: &str,
    code: &str,
    remark: Option<&str>,
) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_dict (name, code, remark) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(code)
    .bind(remark)
    .fetch_one(pool)
    .await
}

pub async fn update_dict_fields(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    remark: Option<&str>,
    status: Option<i32>,
) -> sqlx::Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_dict SET name=? WHERE id=?")
            .bind(n).bind(id).execute(pool).await?;
    }
    if let Some(d) = remark {
        sqlx::query("UPDATE sys_dict SET remark=? WHERE id=?")
            .bind(d).bind(id).execute(pool).await?;
    }
    if let Some(s) = status {
        sqlx::query("UPDATE sys_dict SET status=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_dict(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_dict WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}

pub async fn list_dict_items(pool: &SqlitePool, dict_id: i64) -> sqlx::Result<Vec<DictItem>> {
    sqlx::query_as::<_, DictItem>(
        "SELECT * FROM sys_dict_item WHERE dict_id=? ORDER BY sort, id",
    )
    .bind(dict_id)
    .fetch_all(pool)
    .await
}

pub async fn insert_dict_item_returning_id(
    pool: &SqlitePool,
    dict_id: i64,
    label: &str,
    value: &str,
    sort: i32,
) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_dict_item (dict_id, label, value, sort) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(dict_id)
    .bind(label)
    .bind(value)
    .bind(sort)
    .fetch_one(pool)
    .await
}

pub async fn update_dict_item_fields(
    pool: &SqlitePool,
    id: i64,
    label: Option<&str>,
    value: Option<&str>,
    sort: Option<i32>,
    status: Option<i32>,
) -> sqlx::Result<()> {
    if let Some(l) = label {
        sqlx::query("UPDATE sys_dict_item SET label=? WHERE id=?")
            .bind(l).bind(id).execute(pool).await?;
    }
    if let Some(v) = value {
        sqlx::query("UPDATE sys_dict_item SET value=? WHERE id=?")
            .bind(v).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_dict_item SET sort=? WHERE id=?")
            .bind(s).bind(id).execute(pool).await?;
    }
    if let Some(st) = status {
        sqlx::query("UPDATE sys_dict_item SET status=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(st).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_dict_item(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_dict_item WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}