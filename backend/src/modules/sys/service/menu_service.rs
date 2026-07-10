use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::sys::entity::menu::Menu;
use crate::modules::sys::dao::menu_dao;

pub async fn list_menus(pool: &SqlitePool) -> Result<Vec<Menu>> {
    Ok(menu_dao::list_menus(pool).await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_menu(
    pool: &SqlitePool,
    name: &str,
    title: &str,
    parent_id: Option<i64>,
    icon: Option<&str>,
    path: Option<&str>,
    component: Option<&str>,
    menu_type: i32,
    permission: Option<&str>,
    sort: i32,
) -> Result<i64> {
    Ok(menu_dao::insert_menu_returning_id(
        pool, name, title, parent_id, icon, path, component, menu_type, permission, sort,
    )
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_menu(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    title: Option<&str>,
    parent_id: Option<Option<i64>>,
    icon: Option<Option<&str>>,
    path: Option<Option<&str>>,
    component: Option<Option<&str>>,
    permission: Option<Option<&str>>,
    sort: Option<i32>,
) -> Result<()> {
    Ok(menu_dao::update_menu_fields(
        pool, id, name, title, parent_id, icon, path, component, permission, sort,
    )
    .await?)
}

/// 批量删除菜单：递归收子 + 清关联
pub async fn delete_menus(pool: &SqlitePool, ids: &[i64]) -> Result<usize> {
    Ok(menu_dao::delete_menus(pool, ids).await?)
}

/// 单删菜单
pub async fn delete_menu(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(menu_dao::delete_menus(pool, &[id]).await? > 0)
}