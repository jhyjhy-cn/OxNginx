use anyhow::Result;
use sqlx::SqlitePool;

use crate::model::I18nEntry;

// ============== 国际化 CRUD ==============

pub async fn list_i18n_locales(pool: &SqlitePool) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar("SELECT DISTINCT locale FROM sys_i18n ORDER BY locale")
        .fetch_all(pool)
        .await?)
}

pub async fn list_i18n(pool: &SqlitePool, locale: Option<&str>) -> Result<Vec<I18nEntry>> {
    match locale {
        Some(l) => Ok(sqlx::query_as::<_, I18nEntry>(
            "SELECT * FROM sys_i18n WHERE locale = ? ORDER BY key",
        )
        .bind(l)
        .fetch_all(pool)
        .await?),
        None => Ok(sqlx::query_as::<_, I18nEntry>("SELECT * FROM sys_i18n ORDER BY locale, key")
            .fetch_all(pool)
            .await?),
    }
}

/// 分页查询 i18n，支持 key 模糊搜索
pub async fn list_i18n_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    key: Option<&str>,
) -> Result<(Vec<I18nEntry>, i64)> {
    let offset = (page - 1).max(0) * page_size;
    let like = key.map(|k| format!("%{}%", k));

    let (total, rows) = if let Some(ref pattern) = like {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sys_i18n WHERE key LIKE ?",
        )
        .bind(pattern)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, I18nEntry>(
            "SELECT * FROM sys_i18n WHERE key LIKE ? ORDER BY key, locale LIMIT ? OFFSET ?",
        )
        .bind(pattern)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_i18n")
            .fetch_one(pool)
            .await?;

        let rows = sqlx::query_as::<_, I18nEntry>(
            "SELECT * FROM sys_i18n ORDER BY key, locale LIMIT ? OFFSET ?",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    };

    Ok((rows, total))
}

pub async fn upsert_i18n_batch(
    pool: &SqlitePool,
    locale: &str,
    entries: &[(String, String)],
) -> Result<()> {
    for (key, value) in entries {
        sqlx::query(
            "INSERT INTO sys_i18n (locale, key, value) VALUES (?, ?, ?)
             ON CONFLICT(locale, key) DO UPDATE SET value = ?, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(locale)
        .bind(key)
        .bind(value)
        .bind(value)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn delete_i18n(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_i18n WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn get_i18n_messages(pool: &SqlitePool, locale: &str) -> Result<serde_json::Value> {
    let entries = list_i18n(pool, Some(locale)).await?;
    let mut map = serde_json::Map::new();
    for e in entries {
        let parts: Vec<&str> = e.key.split('.').collect();
        insert_nested(&mut map, &parts, serde_json::Value::String(e.value));
    }
    Ok(serde_json::Value::Object(map))
}

fn insert_nested(
    map: &mut serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
    value: serde_json::Value,
) {
    if keys.len() == 1 {
        map.insert(keys[0].to_string(), value);
        return;
    }
    let entry = map.entry(keys[0].to_string()).or_insert_with(|| {
        serde_json::Value::Object(serde_json::Map::new())
    });
    if let serde_json::Value::Object(ref mut inner) = entry {
        insert_nested(inner, &keys[1..], value);
    }
}
