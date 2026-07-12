use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::modules::sys::entity::dept::Dept;
use crate::modules::sys::dao::dept_dao;

/// 部门树节点
#[derive(Debug, Serialize, Clone)]
pub struct DeptNode {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort: i32,
    pub status: i32, // 1=启用 0=禁用
    pub children: Vec<DeptNode>,
}

/// 分页查询部门
pub async fn list_depts_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> Result<(Vec<Dept>, i64)> {
    Ok(dept_dao::list_depts_paged(pool, page, page_size, keyword).await?)
}

/// 全量部门列表（给树结构用）
pub async fn list_depts(pool: &SqlitePool) -> Result<Vec<Dept>> {
    Ok(dept_dao::list_depts(pool).await?)
}

/// 部门树（嵌套结构）
pub async fn list_dept_tree(pool: &SqlitePool) -> Result<Vec<DeptNode>> {
    let depts = list_depts(pool).await?;
    Ok(build_dept_tree(depts))
}

fn build_dept_tree(depts: Vec<Dept>) -> Vec<DeptNode> {
    fn to_node(d: &Dept) -> DeptNode {
        DeptNode {
            id: d.id,
            parent_id: d.parent_id,
            name: d.name.clone(),
            sort: d.sort,
            status: d.status.clone(),
            children: Vec::new(),
        }
    }

    let mut roots: Vec<DeptNode> = Vec::new();
    let mut all = depts.clone();

    let mut i = 0;
    while i < all.len() {
        if all[i].parent_id.is_none() {
            roots.push(to_node(&all.remove(i)));
        } else {
            i += 1;
        }
    }

    let mut placed = true;
    while placed && !all.is_empty() {
        placed = false;
        let mut i = 0;
        while i < all.len() {
            // ponytail: 第一轮 (line 55) 已把所有 parent_id=None 抽到 roots，此处必然 Some
            let pid = all[i].parent_id.expect("第一轮已过滤 None 节点");
            if attach_child(&mut roots, pid, to_node(&all.remove(i))) {
                placed = true;
            } else {
                i += 1;
            }
        }
    }

    fn attach_child(nodes: &mut Vec<DeptNode>, pid: i64, child: DeptNode) -> bool {
        for n in nodes.iter_mut() {
            if n.id == pid {
                n.children.push(child);
                return true;
            }
            if attach_child(&mut n.children, pid, child.clone()) {
                return true;
            }
        }
        false
    }

    roots
}

pub async fn create_dept(pool: &SqlitePool, name: &str, parent_id: Option<i64>, sort: i32) -> Result<i64> {
    Ok(dept_dao::insert_dept_returning_id(pool, name, parent_id, sort).await?)
}

pub async fn update_dept(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    parent_id: Option<Option<i64>>,
    sort: Option<i32>,
) -> Result<()> {
    Ok(dept_dao::update_dept_fields(pool, id, name, parent_id, sort).await?)
}

pub async fn delete_dept(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(dept_dao::delete_dept(pool, id).await? > 0)
}

pub async fn delete_depts(pool: &SqlitePool, ids: &[i64]) -> Result<usize> {
    Ok(dept_dao::delete_depts(pool, ids).await? as usize)
}