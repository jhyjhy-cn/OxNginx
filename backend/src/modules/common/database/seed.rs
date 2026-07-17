use super::seed_menu::seed_menus;
use anyhow::Result;
use sqlx::SqlitePool;

/// 启动种子：菜单 + super_admin 角色 + admin 绑定
/// ponytail: 国际化 seed 暂不使用（改前端 ts 兜底），需要时取消下方两行注释并启用 init.sql 建表。
pub async fn run(pool: &SqlitePool) -> Result<()> {
    tracing::info!("    同步菜单...");
    seed_menus(pool).await?;
    // super::seed_i18n::seed_i18n(pool).await?;
    Ok(())
}
