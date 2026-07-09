use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::model::Dept;

// ============== 部门 CRUD ==============

/// 部门树节点
#[derive(Debug, Serialize, Clone)]
pub struct DeptNode {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort: i32,
    pub status: String,
    pub children: Vec<DeptNode>,
}

/// 分页查询部门
pub async fn list_depts_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> Result<(Vec<Dept>, i64)> {
    let offset = (page - 1).max(0) * page_size;
    let like = keyword.map(|k| format!("%{}%", k));
    let (total, rows) = if let Some(ref pattern) = like {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sys_depts WHERE name LIKE ?",
        )
        .bind(pattern)
        .fetch_one(pool)
        .await?;
        let rows = sqlx::query_as::<_, Dept>(
            "SELECT * FROM sys_depts WHERE name LIKE ? ORDER BY sort, id LIMIT ? OFFSET ?",
        )
        .bind(pattern)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    } else {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sys_depts")
                .fetch_one(pool)
                .await?;
        let rows = sqlx::query_as::<_, Dept>(
            "SELECT * FROM sys_depts ORDER BY sort, id LIMIT ? OFFSET ?",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    };
    Ok((rows, total))
}

/// 全量部门列表（给树结构用）
pub async fn list_depts(pool: &SqlitePool) -> Result<Vec<Dept>> {
    Ok(sqlx::query_as::<_, Dept>(
        "SELECT * FROM sys_depts ORDER BY sort, id",
    )
    .fetch_all(pool)
    .await?)
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
            let pid = all[i].parent_id.unwrap();
            if attach_child(&mut roots, pid, to_node(&all[i])) {
                all.remove(i);
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
    Ok(sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_depts (name, parent_id, sort) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(parent_id)
    .bind(sort)
    .fetch_one(pool)
    .await?)
}

pub async fn update_dept(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    parent_id: Option<Option<i64>>,
    sort: Option<i32>,
) -> Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_depts SET name=? WHERE id=?")
            .bind(n).bind(id).execute(pool).await?;
    }
    if let Some(p) = parent_id {
        sqlx::query("UPDATE sys_depts SET parent_id=? WHERE id=?")
            .bind(p).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_depts SET sort=? WHERE id=?")
            .bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_dept(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_depts WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}
