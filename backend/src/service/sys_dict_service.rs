use anyhow::Result;
use sqlx::SqlitePool;

use crate::model::{Dict, DictItem};
use crate::dto::DictWithItems;

// ============== 字典 CRUD ==============

pub async fn list_dicts(pool: &SqlitePool) -> Result<Vec<Dict>> {
    Ok(sqlx::query_as::<_, Dict>("SELECT * FROM sys_dict ORDER BY id")
        .fetch_all(pool)
        .await?)
}

pub async fn get_dict(pool: &SqlitePool, id: i64) -> Result<Option<Dict>> {
    Ok(sqlx::query_as::<_, Dict>("SELECT * FROM sys_dict WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn create_dict(
    pool: &SqlitePool,
    name: &str,
    code: &str,
    description: Option<&str>,
) -> Result<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_dict (name, code, description) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(code)
    .bind(description)
    .fetch_one(pool)
    .await?)
}

pub async fn update_dict(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    description: Option<&str>,
    status: Option<&str>,
) -> Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_dict SET name=? WHERE id=?")
            .bind(n).bind(id).execute(pool).await?;
    }
    if let Some(d) = description {
        sqlx::query("UPDATE sys_dict SET description=? WHERE id=?")
            .bind(d).bind(id).execute(pool).await?;
    }
    if let Some(s) = status {
        sqlx::query("UPDATE sys_dict SET status=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_dict(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_dict WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn list_dict_items(pool: &SqlitePool, dict_id: i64) -> Result<Vec<DictItem>> {
    Ok(sqlx::query_as::<_, DictItem>(
        "SELECT * FROM sys_dict_item WHERE dict_id=? ORDER BY sort, id",
    )
    .bind(dict_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_dict_with_items(pool: &SqlitePool, id: i64) -> Result<Option<DictWithItems>> {
    let dict = get_dict(pool, id).await?;
    match dict {
        Some(d) => {
            let items = list_dict_items(pool, id).await?;
            Ok(Some(DictWithItems {
                id: d.id,
                name: d.name,
                code: d.code,
                description: d.description,
                status: d.status,
                items,
            }))
        }
        None => Ok(None),
    }
}

pub async fn create_dict_item(
    pool: &SqlitePool,
    dict_id: i64,
    label: &str,
    value: &str,
    sort: i32,
) -> Result<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_dict_item (dict_id, label, value, sort) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(dict_id)
    .bind(label)
    .bind(value)
    .bind(sort)
    .fetch_one(pool)
    .await?)
}

pub async fn update_dict_item(
    pool: &SqlitePool,
    id: i64,
    label: Option<&str>,
    value: Option<&str>,
    sort: Option<i32>,
    status: Option<&str>,
) -> Result<()> {
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

pub async fn delete_dict_item(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_dict_item WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}
