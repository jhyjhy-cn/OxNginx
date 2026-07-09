use sqlx::SqlitePool;

use crate::modules::sys::entity::i18n::I18nEntry;

pub async fn list_i18n_locales(pool: &SqlitePool) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar("SELECT DISTINCT locale FROM sys_i18n ORDER BY locale")
        .fetch_all(pool)
        .await
}

pub async fn list_i18n_by_locale(pool: &SqlitePool, locale: &str) -> sqlx::Result<Vec<I18nEntry>> {
    sqlx::query_as::<_, I18nEntry>("SELECT * FROM sys_i18n WHERE locale = ? ORDER BY key")
        .bind(locale)
        .fetch_all(pool)
        .await
}

pub async fn list_all_i18n(pool: &SqlitePool) -> sqlx::Result<Vec<I18nEntry>> {
    sqlx::query_as::<_, I18nEntry>("SELECT * FROM sys_i18n ORDER BY locale, key")
        .fetch_all(pool)
        .await
}

pub async fn list_i18n_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    key: Option<&str>,
) -> sqlx::Result<(Vec<I18nEntry>, i64)> {
    let offset = (page - 1).max(0) * page_size;
    let like = key.map(|k| format!("%{}%", k));

    if let Some(ref pattern) = like {
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
        Ok((rows, total))
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
        Ok((rows, total))
    }
}

pub async fn upsert_i18n_entry(
    pool: &SqlitePool,
    locale: &str,
    key: &str,
    value: &str,
) -> sqlx::Result<()> {
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
    Ok(())
}

pub async fn delete_i18n(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_i18n WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}