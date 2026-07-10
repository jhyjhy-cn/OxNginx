use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::sys::entity::role::Role;
use crate::modules::sys::dao::role_dao;

/// 分页查询角色
pub async fn list_roles_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> Result<(Vec<Role>, i64)> {
    Ok(role_dao::list_roles_paged(pool, page, page_size, keyword).await?)
}

pub async fn create_role(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    remark: Option<&str>,
) -> Result<i64> {
    Ok(role_dao::insert_role_returning_id(pool, code, name, remark).await?)
}

pub async fn update_role(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    remark: Option<&str>,
    status: Option<i32>,
) -> Result<()> {
    Ok(role_dao::update_role_fields(pool, id, name, remark, status).await?)
}

pub async fn delete_role(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(role_dao::delete_role_protect_super_admin(pool, id).await? > 0)
}

pub async fn delete_roles(pool: &SqlitePool, ids: &[i64]) -> Result<usize> {
    Ok(role_dao::delete_roles_protect_super_admin(pool, ids).await? as usize)
}

pub async fn set_role_menus(pool: &SqlitePool, role_id: i64, menu_ids: &[i64]) -> Result<()> {
    Ok(role_dao::replace_role_menus(pool, role_id, menu_ids).await?)
}

pub async fn get_role_menus(pool: &SqlitePool, role_id: i64) -> Result<Vec<i64>> {
    Ok(role_dao::list_role_menu_ids(pool, role_id).await?)
}