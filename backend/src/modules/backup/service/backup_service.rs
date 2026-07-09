use crate::modules::backup::dao::backup_dao;
use crate::modules::backup::entity::backup::Backup;
use crate::AppState;

/// 创建备份
pub async fn create_backup(state: &AppState, site_id: i64, config: &str) -> anyhow::Result<Backup> {
    let max_version = backup_dao::max_version_by_site(state.db.pool(), site_id).await?;
    Ok(backup_dao::insert_backup_returning(state.db.pool(), site_id, max_version + 1, config).await?)
}

pub async fn get_backups(state: &AppState, site_id: i64) -> anyhow::Result<Vec<Backup>> {
    Ok(backup_dao::list_backups_by_site(state.db.pool(), site_id).await?)
}

pub async fn restore_backup(state: &AppState, backup_id: i64) -> anyhow::Result<Option<Backup>> {
    Ok(backup_dao::find_backup_by_id(state.db.pool(), backup_id).await?)
}

pub async fn delete_backup(state: &AppState, backup_id: i64) -> anyhow::Result<bool> {
    Ok(backup_dao::delete_backup(state.db.pool(), backup_id).await? > 0)
}