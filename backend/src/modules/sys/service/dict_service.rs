use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::sys::entity::dict::Dict;
use crate::modules::common::dto::DictWithItems;
use crate::modules::sys::dao::dict_dao;

pub async fn list_dicts(pool: &SqlitePool) -> Result<Vec<Dict>> {
    Ok(dict_dao::list_dicts(pool).await?)
}


pub async fn create_dict(
    pool: &SqlitePool,
    name: &str,
    code: &str,
    remark: Option<&str>,
) -> Result<i64> {
    Ok(dict_dao::insert_dict_returning_id(pool, name, code, remark).await?)
}

pub async fn update_dict(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    remark: Option<&str>,
    status: Option<i32>,
) -> Result<()> {
    Ok(dict_dao::update_dict_fields(pool, id, name, remark, status).await?)
}

pub async fn delete_dict(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(dict_dao::delete_dict(pool, id).await? > 0)
}


pub async fn get_dict_with_items(pool: &SqlitePool, id: i64) -> Result<Option<DictWithItems>> {
    let dict = dict_dao::find_dict_by_id(pool, id).await?;
    match dict {
        Some(d) => {
            let items = dict_dao::list_dict_items(pool, id).await?;
            Ok(Some(DictWithItems {
                id: d.id,
                name: d.name,
                code: d.code,
                remark: d.remark,
                status: d.status,
                items,
            }))
        }
        None => Ok(None),
    }
}

pub async fn create_dict_item(
    pool: &SqlitePool,
    dict_id: i64,
    label: &str,
    value: &str,
    sort: i32,
) -> Result<i64> {
    Ok(dict_dao::insert_dict_item_returning_id(pool, dict_id, label, value, sort).await?)
}

pub async fn update_dict_item(
    pool: &SqlitePool,
    id: i64,
    label: Option<&str>,
    value: Option<&str>,
    sort: Option<i32>,
    status: Option<i32>,
) -> Result<()> {
    Ok(dict_dao::update_dict_item_fields(pool, id, label, value, sort, status).await?)
}

pub async fn delete_dict_item(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(dict_dao::delete_dict_item(pool, id).await? > 0)
}