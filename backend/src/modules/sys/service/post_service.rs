use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::sys::entity::post::Post;
use crate::modules::sys::dao::post_dao;

/// 分页查询岗位
pub async fn list_posts_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> Result<(Vec<Post>, i64)> {
    Ok(post_dao::list_posts_paged(pool, page, page_size, keyword).await?)
}

pub async fn create_post(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    sort: i32,
) -> Result<i64> {
    Ok(post_dao::insert_post_returning_id(pool, code, name, sort).await?)
}

pub async fn update_post(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    sort: Option<i32>,
) -> Result<()> {
    Ok(post_dao::update_post_fields(pool, id, name, sort).await?)
}

pub async fn delete_post(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(post_dao::delete_post(pool, id).await? > 0)
}