use sqlx::SqlitePool;

use crate::modules::backup::entity::backup::Backup;

pub async fn max_version_by_site(pool: &SqlitePool, site_id: i64) -> sqlx::Result<i32> {
    sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM site_backups WHERE site_id = ?")
        .bind(site_id)
        .fetch_one(pool)
        .await
}

pub async fn insert_backup_returning(
    pool: &SqlitePool,
    site_id: i64,
    version: i32,
    config: &str,
) -> sqlx::Result<Backup> {
    sqlx::query_as::<_, Backup>(
        r#"
        INSERT INTO site_backups (site_id, version, config)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(site_id)
    .bind(version)
    .bind(config)
    .fetch_one(pool)
    .await
}

pub async fn list_backups_by_site(pool: &SqlitePool, site_id: i64) -> sqlx::Result<Vec<Backup>> {
    sqlx::query_as::<_, Backup>("SELECT * FROM site_backups WHERE site_id = ? ORDER BY version DESC")
        .bind(site_id)
        .fetch_all(pool)
        .await
}

pub async fn find_backup_by_id(pool: &SqlitePool, backup_id: i64) -> sqlx::Result<Option<Backup>> {
    sqlx::query_as::<_, Backup>("SELECT * FROM site_backups WHERE id = ?")
        .bind(backup_id)
        .fetch_optional(pool)
        .await
}

pub async fn delete_backup(pool: &SqlitePool, backup_id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM site_backups WHERE id = ?")
        .bind(backup_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}