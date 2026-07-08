use super::seed_i18n::seed_i18n;
use super::seed_menu::seed_menus;
use anyhow::Result;
use sqlx::SqlitePool;

/// 启动种子：菜单 + super_admin 角色 + admin 绑定 + i18n
pub async fn run(pool: &SqlitePool) -> Result<()> {
    seed_menus(pool).await?;
    seed_i18n(pool).await?;
    Ok(())
}
