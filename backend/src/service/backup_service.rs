use crate::model::Backup;
use crate::AppState;

/// 创建备份
pub async fn create_backup(state: &AppState, site_id: i64, config: &str) -> anyhow::Result<Backup> {
    // 获取当前最新版本号
    let max_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM sys_backups WHERE site_id = ?",
    )
    .bind(site_id)
    .fetch_one(state.db.pool())
    .await?;

    let backup = sqlx::query_as::<_, Backup>(
        r#"
        INSERT INTO sys_backups (site_id, version, config)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(site_id)
    .bind(max_version + 1)
    .bind(config)
    .fetch_one(state.db.pool())
    .await?;

    Ok(backup)
}

/// 获取站点的备份列表
pub async fn get_backups(state: &AppState, site_id: i64) -> anyhow::Result<Vec<Backup>> {
    let backups = sqlx::query_as::<_, Backup>(
        "SELECT * FROM sys_backups WHERE site_id = ? ORDER BY version DESC",
    )
    .bind(site_id)
    .fetch_all(state.db.pool())
    .await?;

    Ok(backups)
}

/// 恢复备份
pub async fn restore_backup(state: &AppState, backup_id: i64) -> anyhow::Result<Option<Backup>> {
    let backup = sqlx::query_as::<_, Backup>("SELECT * FROM sys_backups WHERE id = ?")
        .bind(backup_id)
        .fetch_optional(state.db.pool())
        .await?;

    Ok(backup)
}

/// 删除备份
pub async fn delete_backup(state: &AppState, backup_id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM sys_backups WHERE id = ?")
        .bind(backup_id)
        .execute(state.db.pool())
        .await?;

    Ok(result.rows_affected() > 0)
}
