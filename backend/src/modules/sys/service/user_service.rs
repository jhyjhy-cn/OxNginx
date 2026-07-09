use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::modules::common::dto::{MenuNode, UserQuery};
use crate::modules::sys::dao::user_dao;
use crate::modules::sys::entity::user::User;
use crate::modules::sys::entity::menu::Menu;

/// RBAC 信息（roles + 权限码 + 菜单树）
pub async fn get_rbac_info(pool: &SqlitePool, username: &str) -> Result<(Vec<String>, Vec<String>, Vec<MenuNode>)> {
    if username == "admin" {
        let roles = vec!["super_admin".to_string()];
        let perms = user_dao::list_all_enabled_permissions(pool).await?;
        let menus = all_menu_tree(pool).await?;
        return Ok((roles, perms, menus));
    }

    let roles = get_user_roles(pool, username).await?;
    let perms = get_user_permissions(pool, username).await?;
    let menus = get_user_menu_tree(pool, username).await?;
    Ok((roles, perms, menus))
}

pub async fn user_is_super_admin(pool: &SqlitePool, username: &str) -> Result<bool> {
    Ok(user_dao::count_super_admin_by_username(pool, username).await? > 0)
}

async fn get_user_roles(pool: &SqlitePool, username: &str) -> Result<Vec<String>> {
    Ok(user_dao::list_enabled_role_codes_by_username(pool, username).await?)
}

async fn get_user_permissions(pool: &SqlitePool, username: &str) -> Result<Vec<String>> {
    Ok(user_dao::list_user_permissions(pool, username).await?)
}

async fn all_menu_tree(pool: &SqlitePool) -> Result<Vec<MenuNode>> {
    let menus = user_dao::list_all_enabled_menus(pool).await?;
    Ok(build_menu_tree(menus))
}

async fn get_user_menu_tree(pool: &SqlitePool, username: &str) -> Result<Vec<MenuNode>> {
    let menus = user_dao::list_user_menus(pool, username).await?;
    Ok(build_menu_tree(menus))
}

/// 拼菜单树：parent_id=None 为根，按 parent_id 嵌套
fn build_menu_tree(menus: Vec<Menu>) -> Vec<MenuNode> {
    fn to_node(m: &Menu) -> MenuNode {
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

    let total = user_dao::count_users_paged(pool, &where_clause, &binds, &int_binds, &dis_binds).await?;

    let items = user_dao::list_users_paged(
        pool,
        &where_clause,
        &binds,
        &int_binds,
        &dis_binds,
        page_size,
        offset,
    )
    .await?;

    Ok((items, total))
}

/// 创建用户
pub async fn create_user(pool: &SqlitePool, username: &str, password: &str) -> Result<i64> {
    let hashed = crate::modules::common::auth::hash_password(password)?;
    Ok(user_dao::insert_user_returning_id(pool, username, &hashed).await?)
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
    user_dao::update_user_fields(
        pool,
        id,
        nickname,
        phone,
        email,
        gender,
        remark,
        dept_id,
        post_id,
        disabled,
    )
    .await?;
    if let Some(rids) = role_ids {
        user_dao::replace_user_roles(pool, id, &rids).await?;
    }
    Ok(())
}

/// 删除用户（保护 admin 不允许删除）
pub async fn delete_user(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(user_dao::delete_user_protect_admin(pool, id).await? > 0)
}

/// 重置密码（admin 用户可被重置）
pub async fn reset_password(pool: &SqlitePool, id: i64, new_password: &str) -> Result<()> {
    let hashed = crate::modules::common::auth::hash_password(new_password)?;
    user_dao::update_user_password(pool, id, &hashed).await?;
    Ok(())
}

/// 检查是否为 admin 用户
pub async fn is_admin_user(pool: &SqlitePool, id: i64) -> Result<bool> {
    Ok(user_dao::count_admin_by_id(pool, id).await? > 0)
}

/// 获取用户详情
pub async fn get_user(pool: &SqlitePool, id: i64) -> Result<Option<User>> {
    Ok(user_dao::find_user_by_id(pool, id).await?)
}

/// 获取用户的角色 ID 列表
pub async fn get_user_role_ids(pool: &SqlitePool, user_id: i64) -> Result<Vec<i64>> {
    Ok(user_dao::list_user_role_ids(pool, user_id).await?)
}