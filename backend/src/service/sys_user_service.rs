use anyhow::Result;
use serde::Serialize;
use sqlx::{Row, SqlitePool};

use crate::auth;
use crate::dto::{MenuNode, UserQuery};

// ============== RBAC 信息（给 /api/rbac/me 用）==============

/// 当前用户的 RBAC 信息（roles + 权限码 + 菜单树）
pub async fn get_rbac_info(pool: &SqlitePool, username: &str) -> Result<(Vec<String>, Vec<String>, Vec<MenuNode>)> {
    if username == "admin" {
        let roles = vec!["super_admin".to_string()];
        let perms = sqlx::query_scalar("SELECT permission FROM sys_menus WHERE permission IS NOT NULL AND status='enabled'")
            .fetch_all(pool)
            .await?;
        let menus = all_menu_tree(pool).await?;
        return Ok((roles, perms, menus));
    }

    let roles = get_user_roles(pool, username).await?;
    let perms = get_user_permissions(pool, username).await?;
    let menus = get_user_menu_tree(pool, username).await?;
    Ok((roles, perms, menus))
}

pub async fn user_is_super_admin(pool: &SqlitePool, username: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sys_user_roles ur
         JOIN sys_roles r ON ur.role_id = r.id
         WHERE r.code = 'super_admin'
           AND ur.user_id = (SELECT id FROM sys_users WHERE username = ?)",
    )
    .bind(username)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

async fn get_user_roles(pool: &SqlitePool, username: &str) -> Result<Vec<String>> {
    let codes: Vec<String> = sqlx::query_scalar(
        "SELECT r.code FROM sys_roles r
         JOIN sys_user_roles ur ON r.id = ur.role_id
         JOIN sys_users u ON ur.user_id = u.id
         WHERE u.username = ? AND r.status='enabled'",
    )
    .bind(username)
    .fetch_all(pool)
    .await?;
    Ok(codes)
}

async fn get_user_permissions(pool: &SqlitePool, username: &str) -> Result<Vec<String>> {
    let perms: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT m.permission FROM sys_menus m
         JOIN sys_role_menus rm ON m.id = rm.menu_id
         JOIN sys_user_roles ur ON rm.role_id = ur.role_id
         JOIN sys_users u ON ur.user_id = u.id
         WHERE u.username = ? AND m.permission IS NOT NULL AND m.status='enabled'",
    )
    .bind(username)
    .fetch_all(pool)
    .await?;
    Ok(perms)
}

async fn all_menu_tree(pool: &SqlitePool) -> Result<Vec<MenuNode>> {
    let menus = sqlx::query_as::<_, crate::model::Menu>(
        "SELECT id, parent_id, name, title, icon, path, component, type, permission, sort, status, created_at, updated_at
         FROM sys_menus WHERE status='enabled' ORDER BY sort, id",
    )
    .fetch_all(pool)
    .await?;
    Ok(build_menu_tree(menus))
}

async fn get_user_menu_tree(pool: &SqlitePool, username: &str) -> Result<Vec<MenuNode>> {
    let menus = sqlx::query_as::<_, crate::model::Menu>(
        "SELECT DISTINCT m.id, m.parent_id, m.name, m.title, m.icon, m.path, m.component, m.type, m.permission, m.sort, m.status, m.created_at, m.updated_at
         FROM sys_menus m
         JOIN sys_role_menus rm ON m.id = rm.menu_id
         JOIN sys_user_roles ur ON rm.role_id = ur.role_id
         JOIN sys_users u ON ur.user_id = u.id
         WHERE u.username = ? AND m.status='enabled'
         ORDER BY m.sort, m.id",
    )
    .bind(username)
    .fetch_all(pool)
    .await?;
    Ok(build_menu_tree(menus))
}

/// 拼菜单树：parent_id=None 为根，按 parent_id 嵌套
fn build_menu_tree(menus: Vec<crate::model::Menu>) -> Vec<MenuNode> {
    fn to_node(m: &crate::model::Menu) -> MenuNode {
        MenuNode {
            id: m.id,
            parent_id: m.parent_id,
            name: m.name.clone(),
            title: m.title.clone(),
            icon: m.icon.clone(),
            path: m.path.clone(),
            component: m.component.clone(),
            menu_type: m.menu_type.clone(),
            permission: m.permission.clone(),
            sort: m.sort,
            children: Vec::new(),
        }
    }

    let mut roots: Vec<MenuNode> = Vec::new();
    let mut all = menus.clone();

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

    fn attach_child(nodes: &mut Vec<MenuNode>, pid: i64, child: MenuNode) -> bool {
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

// ============== 用户 CRUD ==============

/// 用户列表项（含部门名、岗位名）
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserListItem {
    pub id: i64,
    pub username: String,
    pub nickname: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub remark: Option<String>,
    pub dept_id: Option<i64>,
    pub dept_name: Option<String>,
    pub post_id: Option<i64>,
    pub post_name: Option<String>,
    pub disabled: i32,
    pub created_at: Option<chrono::NaiveDateTime>,
}

/// 分页查询用户列表
pub async fn list_users_paged(
    pool: &SqlitePool,
    query: &UserQuery,
) -> Result<(Vec<UserListItem>, i64)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1);
    let offset = (page - 1).max(0) * page_size;

    let mut where_clause = String::from(" WHERE 1=1");
    let mut binds: Vec<String> = Vec::new();
    let mut int_binds: Vec<i64> = Vec::new();
    let mut dis_binds: Vec<i32> = Vec::new();
    if let Some(kw) = &query.keyword {
        where_clause.push_str(" AND (u.username LIKE ? OR u.nickname LIKE ?)");
        binds.push(format!("%{}%", kw));
        binds.push(format!("%{}%", kw));
    } else if let Some(un) = &query.username {
        where_clause.push_str(" AND u.username LIKE ?");
        binds.push(format!("%{}%", un));
    }
    if let Some(d) = query.dept_id {
        where_clause.push_str(" AND u.dept_id=?");
        int_binds.push(d);
    }
    if let Some(p) = &query.phone {
        where_clause.push_str(" AND u.phone LIKE ?");
        binds.push(format!("%{}%", p));
    }
    if let Some(s) = &query.status {
        where_clause.push_str(" AND u.disabled=?");
        dis_binds.push(if s == "enabled" { 0 } else { 1 });
    }
    if let Some(s) = &query.start_date {
        where_clause.push_str(" AND u.created_at>=?");
        binds.push(s.clone());
    }
    if let Some(e) = &query.end_date {
        where_clause.push_str(" AND u.created_at<?");
        binds.push(e.clone());
    }

    // 计数 SQL
    let count_sql = format!(
        "SELECT COUNT(DISTINCT u.id) FROM sys_users u
         LEFT JOIN sys_depts d ON u.dept_id=d.id
         LEFT JOIN sys_posts p ON u.post_id=p.id{}",
        where_clause
    );
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds { count_q = count_q.bind(b); }
    for i in &int_binds { count_q = count_q.bind(*i); }
    for d in &dis_binds { count_q = count_q.bind(*d); }
    let total = count_q.fetch_one(pool).await?;

    // 列表 SQL
    let list_sql = format!(
        "SELECT u.id, u.username, u.nickname, u.phone, u.email, u.gender, u.remark,
                u.dept_id, d.name as dept_name,
                u.post_id, p.name as post_name,
                u.disabled, u.created_at
         FROM sys_users u
         LEFT JOIN sys_depts d ON u.dept_id=d.id
         LEFT JOIN sys_posts p ON u.post_id=p.id{}
         ORDER BY u.id DESC LIMIT ? OFFSET ?",
        where_clause
    );
    let mut list_q = sqlx::query(&list_sql);
    for b in &binds { list_q = list_q.bind(b); }
    for i in &int_binds { list_q = list_q.bind(*i); }
    for d in &dis_binds { list_q = list_q.bind(*d); }
    list_q = list_q.bind(page_size).bind(offset);

    let rows = list_q.fetch_all(pool).await?;
    let items: Vec<UserListItem> = rows
        .into_iter()
        .map(|r| UserListItem {
            id: r.get("id"),
            username: r.get("username"),
            nickname: r.get("nickname"),
            phone: r.get("phone"),
            email: r.get("email"),
            gender: r.get("gender"),
            remark: r.get("remark"),
            dept_id: r.get("dept_id"),
            dept_name: r.get("dept_name"),
            post_id: r.get("post_id"),
            post_name: r.get("post_name"),
            disabled: r.get("disabled"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok((items, total))
}

/// 创建用户
pub async fn create_user(pool: &SqlitePool, username: &str, password: &str) -> Result<i64> {
    let hashed = auth::hash_password(password)?;
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_users (username, password) VALUES (?, ?) RETURNING id",
    )
    .bind(username)
    .bind(&hashed)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// 更新用户基本信息（不含密码、角色）
pub async fn update_user(
    pool: &SqlitePool,
    id: i64,
    nickname: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    gender: Option<&str>,
    remark: Option<&str>,
    dept_id: Option<i64>,
    post_id: Option<i64>,
    disabled: Option<i32>,
    role_ids: Option<Vec<i64>>,
) -> Result<()> {
    if let Some(v) = nickname {
        sqlx::query("UPDATE sys_users SET nickname=? WHERE id=?")
            .bind(v).bind(id).execute(pool).await?;
    }
    if let Some(v) = phone {
        sqlx::query("UPDATE sys_users SET phone=? WHERE id=?")
            .bind(v).bind(id).execute(pool).await?;
    }
    if let Some(v) = email {
        sqlx::query("UPDATE sys_users SET email=? WHERE id=?")
            .bind(v).bind(id).execute(pool).await?;
    }
    if let Some(v) = gender {
        sqlx::query("UPDATE sys_users SET gender=? WHERE id=?")
            .bind(v).bind(id).execute(pool).await?;
    }
    if let Some(v) = remark {
        sqlx::query("UPDATE sys_users SET remark=? WHERE id=?")
            .bind(v).bind(id).execute(pool).await?;
    }
    if let Some(d) = dept_id {
        sqlx::query("UPDATE sys_users SET dept_id=? WHERE id=?")
            .bind(d).bind(id).execute(pool).await?;
    }
    if let Some(p) = post_id {
        sqlx::query("UPDATE sys_users SET post_id=? WHERE id=?")
            .bind(p).bind(id).execute(pool).await?;
    }
    if let Some(dis) = disabled {
        sqlx::query("UPDATE sys_users SET disabled=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(dis).bind(id).execute(pool).await?;
    }
    if let Some(rids) = role_ids {
        sqlx::query("DELETE FROM sys_user_roles WHERE user_id=?")
            .bind(id).execute(pool).await?;
        for rid in rids {
            sqlx::query("INSERT OR IGNORE INTO sys_user_roles (user_id, role_id) VALUES (?, ?)")
                .bind(id).bind(rid).execute(pool).await?;
        }
    }
    Ok(())
}

/// 删除用户（保护 admin 不允许删除）
pub async fn delete_user(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_users WHERE id=? AND username != 'admin'")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

/// 重置密码（admin 用户可被重置）
pub async fn reset_password(pool: &SqlitePool, id: i64, new_password: &str) -> Result<()> {
    let hashed = auth::hash_password(new_password)?;
    sqlx::query("UPDATE sys_users SET password=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(&hashed)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 检查是否为 admin 用户
pub async fn is_admin_user(pool: &SqlitePool, id: i64) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sys_users WHERE id=? AND username='admin'",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

/// 获取用户详情
pub async fn get_user(pool: &SqlitePool, id: i64) -> Result<Option<crate::model::User>> {
    Ok(sqlx::query_as::<_, crate::model::User>("SELECT * FROM sys_users WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// 获取用户的角色 ID 列表
pub async fn get_user_role_ids(pool: &SqlitePool, user_id: i64) -> Result<Vec<i64>> {
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT role_id FROM sys_user_roles WHERE user_id=?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(ids)
}
