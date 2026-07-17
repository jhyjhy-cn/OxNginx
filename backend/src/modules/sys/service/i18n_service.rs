// ============================================================================
// 暂不使用: i18n Service (改用前端 ts 兜底)
// ============================================================================
/*
use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::sys::entity::i18n::I18nEntry;
use crate::modules::sys::dao::i18n_dao;

pub async fn list_i18n_locales(pool: &SqlitePool) -> Result<Vec<String>> {
    Ok(i18n_dao::list_i18n_locales(pool).await?)
}

pub async fn list_i18n(pool: &SqlitePool, locale: Option<&str>) -> Result<Vec<I18nEntry>> {
    match locale {
        Some(l) => Ok(i18n_dao::list_i18n_by_locale(pool, l).await?),
        None => Ok(i18n_dao::list_all_i18n(pool).await?),
    }
}

/// 分页查询 i18n，支持 key 模糊搜索
pub async fn list_i18n_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    key: Option<&str>,
) -> Result<(Vec<I18nEntry>, i64)> {
    Ok(i18n_dao::list_i18n_paged(pool, page, page_size, key).await?)
}

pub async fn upsert_i18n_batch(
    pool: &SqlitePool,
    locale: &str,
    entries: &[(String, String)],
) -> Result<()> {
    for (key, value) in entries {
        i18n_dao::upsert_i18n_entry(pool, locale, key, value).await?;
    }
    Ok(())
}

pub async fn delete_i18n(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(i18n_dao::delete_i18n(pool, id).await? > 0)
}

pub async fn get_i18n_messages(pool: &SqlitePool, locale: &str) -> Result<serde_json::Value> {
    let entries = i18n_dao::list_i18n_by_locale(pool, locale).await?;
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
*/