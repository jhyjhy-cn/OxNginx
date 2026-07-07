// ponytail: 单文件不分 rbac_user/role/menu 子模块；CRUD 一锅端
// ponytail: 按需 DB 查询，未上通用中间件；高频路径加 Arc<DashMap> 缓存时改

use anyhow::Result;
use serde::Serialize;
use sqlx::{Row, SqlitePool};

use crate::auth;
use crate::dto::MenuNode;
use crate::model::{Dept, Dict, DictItem, I18nEntry, Menu, Post, Role, User};

// ============== 用户 RBAC 信息 ==============

/// 当前用户的 RBAC 信息（roles + 权限码 + 菜单树）
pub async fn get_rbac_info(pool: &SqlitePool, username: &str) -> Result<(Vec<String>, Vec<String>, Vec<MenuNode>)> {
    if username == "admin" {
        // ponytail: super_admin 全权限；查菜单表返回所有启用的 C/M/F
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

/// ponytail: 业务级按钮权限按需查 DB
pub async fn check_permission(pool: &SqlitePool, username: &str, perm: &str) -> Result<bool> {
    if username == "admin" {
        return Ok(true);
    }
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sys_menus m
         JOIN sys_role_menus rm ON m.id = rm.menu_id
         JOIN sys_user_roles ur ON rm.role_id = ur.role_id
         JOIN sys_users u ON ur.user_id = u.id
         WHERE u.username = ? AND m.permission = ? AND m.status='enabled'",
    )
    .bind(username)
    .bind(perm)
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
    let menus = sqlx::query_as::<_, Menu>(
        "SELECT id, parent_id, name, title, icon, path, component, type, permission, sort, status, created_at, updated_at
         FROM sys_menus WHERE status='enabled' ORDER BY sort, id",
    )
    .fetch_all(pool)
    .await?;
    Ok(build_tree(menus))
}

async fn get_user_menu_tree(pool: &SqlitePool, username: &str) -> Result<Vec<MenuNode>> {
    let menus = sqlx::query_as::<_, Menu>(
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
    Ok(build_tree(menus))
}

/// 拼树：parent_id=None 为根，按 parent_id 嵌套
fn build_tree(menus: Vec<Menu>) -> Vec<MenuNode> {
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

    // 第一遍：把根挑出来
    let mut i = 0;
    while i < all.len() {
        if all[i].parent_id.is_none() {
            roots.push(to_node(&all.remove(i)));
        } else {
            i += 1;
        }
    }

    // 第二遍：按 parent_id 挂
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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserListItem {
    pub id: i64,
    pub username: String,
    pub dept_id: Option<i64>,
    pub post_id: Option<i64>,
    pub disabled: i32,
    pub status: Option<String>,
    pub roles: String, // 拼接的角色名
    pub created_at: Option<chrono::NaiveDateTime>,
}

pub async fn list_users(pool: &SqlitePool) -> Result<Vec<UserListItem>> {
    let rows = sqlx::query(
        "SELECT u.id, u.username, u.dept_id, u.post_id, u.disabled, u.status, u.created_at,
                GROUP_CONCAT(r.name, ',') AS roles
         FROM sys_users u
         LEFT JOIN sys_user_roles ur ON u.id = ur.user_id
         LEFT JOIN sys_roles r ON ur.role_id = r.id
         GROUP BY u.id
         ORDER BY u.id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| UserListItem {
            id: r.get("id"),
            username: r.get("username"),
            dept_id: r.get("dept_id"),
            post_id: r.get("post_id"),
            disabled: r.get("disabled"),
            status: r.get("status"),
            roles: r.get::<Option<String>, _>("roles").unwrap_or_default(),
            created_at: r.get("created_at"),
        })
        .collect())
}

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

pub async fn update_user(
    pool: &SqlitePool,
    id: i64,
    dept_id: Option<i64>,
    post_id: Option<i64>,
    disabled: Option<i32>,
    role_ids: Option<Vec<i64>>,
) -> Result<()> {
    // ponytail: 动态 SQL 拼一下；参数化绑定避免注入
    if let Some(d) = dept_id {
        sqlx::query("UPDATE sys_users SET dept_id=? WHERE id=?").bind(d).bind(id).execute(pool).await?;
    }
    if let Some(p) = post_id {
        sqlx::query("UPDATE sys_users SET post_id=? WHERE id=?").bind(p).bind(id).execute(pool).await?;
    }
    if let Some(dis) = disabled {
        sqlx::query("UPDATE sys_users SET disabled=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(dis).bind(id).execute(pool).await?;
    }
    if let Some(rids) = role_ids {
        sqlx::query("DELETE FROM sys_user_roles WHERE user_id=?").bind(id).execute(pool).await?;
        for rid in rids {
            sqlx::query("INSERT OR IGNORE INTO sys_user_roles (user_id, role_id) VALUES (?, ?)")
                .bind(id).bind(rid).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn delete_user(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_users WHERE id=? AND username != 'admin'")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn reset_password(pool: &SqlitePool, id: i64, new_password: &str) -> Result<()> {
    let hashed = auth::hash_password(new_password)?;
    sqlx::query("UPDATE sys_users SET password=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(&hashed)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ============== 角色 CRUD ==============

pub async fn list_roles(pool: &SqlitePool) -> Result<Vec<Role>> {
    Ok(sqlx::query_as::<_, Role>("SELECT * FROM sys_roles ORDER BY id")
        .fetch_all(pool)
        .await?)
}

pub async fn create_role(pool: &SqlitePool, code: &str, name: &str, remark: Option<&str>) -> Result<i64> {
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_roles (code, name, remark) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(code)
    .bind(name)
    .bind(remark)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update_role(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    remark: Option<&str>,
    status: Option<&str>,
) -> Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_roles SET name=? WHERE id=?").bind(n).bind(id).execute(pool).await?;
    }
    if let Some(r) = remark {
        sqlx::query("UPDATE sys_roles SET remark=? WHERE id=?").bind(r).bind(id).execute(pool).await?;
    }
    if let Some(s) = status {
        sqlx::query("UPDATE sys_roles SET status=? WHERE id=?").bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_role(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_roles WHERE id=? AND code != 'super_admin'")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn set_role_menus(pool: &SqlitePool, role_id: i64, menu_ids: &[i64]) -> Result<()> {
    sqlx::query("DELETE FROM sys_role_menus WHERE role_id=?").bind(role_id).execute(pool).await?;
    for mid in menu_ids {
        sqlx::query("INSERT OR IGNORE INTO sys_role_menus (role_id, menu_id) VALUES (?, ?)")
            .bind(role_id)
            .bind(mid)
            .execute(pool)
            .await?;
    }
    Ok(())
}

// ============== 部门 CRUD ==============

pub async fn list_depts(pool: &SqlitePool) -> Result<Vec<Dept>> {
    Ok(sqlx::query_as::<_, Dept>("SELECT * FROM sys_depts ORDER BY sort, id")
        .fetch_all(pool)
        .await?)
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

pub async fn update_dept(pool: &SqlitePool, id: i64, name: Option<&str>, parent_id: Option<Option<i64>>, sort: Option<i32>) -> Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_depts SET name=? WHERE id=?").bind(n).bind(id).execute(pool).await?;
    }
    if let Some(p) = parent_id {
        sqlx::query("UPDATE sys_depts SET parent_id=? WHERE id=?").bind(p).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_depts SET sort=? WHERE id=?").bind(s).bind(id).execute(pool).await?;
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

// ============== 岗位 CRUD ==============

pub async fn list_posts(pool: &SqlitePool) -> Result<Vec<Post>> {
    Ok(sqlx::query_as::<_, Post>("SELECT * FROM sys_posts ORDER BY sort, id")
        .fetch_all(pool)
        .await?)
}

pub async fn create_post(pool: &SqlitePool, code: &str, name: &str, sort: i32) -> Result<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_posts (code, name, sort) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(code)
    .bind(name)
    .bind(sort)
    .fetch_one(pool)
    .await?)
}

pub async fn update_post(pool: &SqlitePool, id: i64, name: Option<&str>, sort: Option<i32>) -> Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_posts SET name=? WHERE id=?").bind(n).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_posts SET sort=? WHERE id=?").bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_post(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_posts WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

// ============== 菜单 CRUD ==============

pub async fn list_menus(pool: &SqlitePool) -> Result<Vec<Menu>> {
    Ok(sqlx::query_as::<_, Menu>("SELECT * FROM sys_menus ORDER BY sort, id")
        .fetch_all(pool)
        .await?)
}

pub async fn create_menu(
    pool: &SqlitePool,
    name: &str,
    title: &str,
    parent_id: Option<i64>,
    icon: Option<&str>,
    path: Option<&str>,
    component: Option<&str>,
    menu_type: &str,
    permission: Option<&str>,
    sort: i32,
) -> Result<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_menus (name, title, parent_id, icon, path, component, type, permission, sort)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(title)
    .bind(parent_id)
    .bind(icon)
    .bind(path)
    .bind(component)
    .bind(menu_type)
    .bind(permission)
    .bind(sort)
    .fetch_one(pool)
    .await?)
}

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
    if let Some(n) = name {
        sqlx::query("UPDATE sys_menus SET name=? WHERE id=?").bind(n).bind(id).execute(pool).await?;
    }
    if let Some(t) = title {
        sqlx::query("UPDATE sys_menus SET title=? WHERE id=?").bind(t).bind(id).execute(pool).await?;
    }
    if let Some(p) = parent_id {
        sqlx::query("UPDATE sys_menus SET parent_id=? WHERE id=?").bind(p).bind(id).execute(pool).await?;
    }
    if let Some(i) = icon {
        sqlx::query("UPDATE sys_menus SET icon=? WHERE id=?").bind(i).bind(id).execute(pool).await?;
    }
    if let Some(p) = path {
        sqlx::query("UPDATE sys_menus SET path=? WHERE id=?").bind(p).bind(id).execute(pool).await?;
    }
    if let Some(c) = component {
        sqlx::query("UPDATE sys_menus SET component=? WHERE id=?").bind(c).bind(id).execute(pool).await?;
    }
    if let Some(pm) = permission {
        sqlx::query("UPDATE sys_menus SET permission=? WHERE id=?").bind(pm).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_menus SET sort=? WHERE id=?").bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

/// 批量删除菜单：递归收子 + 清关联
pub async fn delete_menus(pool: &SqlitePool, ids: &[i64]) -> Result<usize> {
    let mut all_ids: Vec<i64> = ids.to_vec();
    let mut frontier = ids.to_vec();
    // 递归收集所有子菜单 id
    while !frontier.is_empty() {
        let placeholders = vec!["?"; frontier.len()].join(",");
        let sql = format!(
            "SELECT id FROM sys_menus WHERE parent_id IN ({})",
            placeholders
        );
        let mut q = sqlx::query_as::<_, (i64,)>(&sql);
        for id in &frontier {
            q = q.bind(id);
        }
        let children: Vec<(i64,)> = q.fetch_all(pool).await?;
        let child_ids: Vec<i64> = children.into_iter().map(|(c,)| c).collect();
        for c in &child_ids {
            if !all_ids.contains(c) {
                all_ids.push(*c);
            }
        }
        frontier = child_ids;
    }

    if all_ids.is_empty() {
        return Ok(0);
    }
    let placeholders = vec!["?"; all_ids.len()].join(",");
    let sql = format!(
        "DELETE FROM sys_role_menus WHERE menu_id IN ({})",
        placeholders
    );
    let mut q = sqlx::query(&sql);
    for id in &all_ids {
        q = q.bind(id);
    }
    q.execute(pool).await?;
    let sql = format!("DELETE FROM sys_menus WHERE id IN ({})", placeholders);
    let mut q = sqlx::query(&sql);
    for id in &all_ids {
        q = q.bind(id);
    }
    let n = q.execute(pool).await?.rows_affected();
    Ok(n as usize)
}

/// 单删菜单(单 id 也走批量,统一入口)
pub async fn delete_menu(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = delete_menus(pool, &[id]).await?;
    Ok(n > 0)
}

/// 从 username + password 直接拿 User（登录用）
pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>> {
    Ok(sqlx::query_as::<_, User>("SELECT * FROM sys_users WHERE username=?")
        .bind(username)
        .fetch_optional(pool)
        .await?)
}

// ============== 国际化 ==============

pub async fn list_i18n_locales(pool: &SqlitePool) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar("SELECT DISTINCT locale FROM sys_i18n ORDER BY locale")
        .fetch_all(pool)
        .await?)
}

pub async fn list_i18n(pool: &SqlitePool, locale: Option<&str>) -> Result<Vec<I18nEntry>> {
    match locale {
        Some(l) => Ok(sqlx::query_as::<_, I18nEntry>(
            "SELECT * FROM sys_i18n WHERE locale = ? ORDER BY key",
        )
        .bind(l)
        .fetch_all(pool)
        .await?),
        None => Ok(sqlx::query_as::<_, I18nEntry>("SELECT * FROM sys_i18n ORDER BY locale, key")
            .fetch_all(pool)
            .await?),
    }
}

/// 分页查询 i18n，支持 key 模糊搜索
pub async fn list_i18n_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    key: Option<&str>,
) -> Result<(Vec<I18nEntry>, i64)> {
    let offset = (page - 1).max(0) * page_size;
    let like = key.map(|k| format!("%{}%", k));

    let (total, rows) = if let Some(ref pattern) = like {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sys_i18n WHERE key LIKE ?",
        )
        .bind(pattern)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query_as::<_, I18nEntry>(
            "SELECT * FROM sys_i18n WHERE key LIKE ? ORDER BY key, locale LIMIT ? OFFSET ?",
        )
        .bind(pattern)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_i18n")
            .fetch_one(pool)
            .await?;

        let rows = sqlx::query_as::<_, I18nEntry>(
            "SELECT * FROM sys_i18n ORDER BY key, locale LIMIT ? OFFSET ?",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    };

    Ok((rows, total))
}

pub async fn upsert_i18n_batch(pool: &SqlitePool, locale: &str, entries: &[(String, String)]) -> Result<()> {
    for (key, value) in entries {
        sqlx::query(
            "INSERT INTO sys_i18n (locale, key, value) VALUES (?, ?, ?)
             ON CONFLICT(locale, key) DO UPDATE SET value = ?, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(locale)
        .bind(key)
        .bind(value)
        .bind(value)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn delete_i18n(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_i18n WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn get_i18n_messages(pool: &SqlitePool, locale: &str) -> Result<serde_json::Value> {
    let entries = list_i18n(pool, Some(locale)).await?;
    let mut map = serde_json::Map::new();
    for e in entries {
        // 'menu.dashboard' -> { "menu": { "dashboard": "..." } }
        let parts: Vec<&str> = e.key.split('.').collect();
        insert_nested(&mut map, &parts, serde_json::Value::String(e.value));
    }
    Ok(serde_json::Value::Object(map))
}

fn insert_nested(map: &mut serde_json::Map<String, serde_json::Value>, keys: &[&str], value: serde_json::Value) {
    if keys.len() == 1 {
        map.insert(keys[0].to_string(), value);
        return;
    }
    let entry = map.entry(keys[0].to_string()).or_insert_with(|| {
        serde_json::Value::Object(serde_json::Map::new())
    });
    if let serde_json::Value::Object(ref mut inner) = entry {
        insert_nested(inner, &keys[1..], value);
    }
}

// ============== 字典 CRUD ==============

pub async fn list_dicts(pool: &SqlitePool) -> Result<Vec<Dict>> {
    Ok(sqlx::query_as::<_, Dict>("SELECT * FROM sys_dict ORDER BY id")
        .fetch_all(pool)
        .await?)
}

pub async fn get_dict(pool: &SqlitePool, id: i64) -> Result<Option<Dict>> {
    Ok(sqlx::query_as::<_, Dict>("SELECT * FROM sys_dict WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn create_dict(pool: &SqlitePool, name: &str, code: &str, description: Option<&str>) -> Result<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_dict (name, code, description) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(code)
    .bind(description)
    .fetch_one(pool)
    .await?)
}

pub async fn update_dict(pool: &SqlitePool, id: i64, name: Option<&str>, description: Option<&str>, status: Option<&str>) -> Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_dict SET name=? WHERE id=?").bind(n).bind(id).execute(pool).await?;
    }
    if let Some(d) = description {
        sqlx::query("UPDATE sys_dict SET description=? WHERE id=?").bind(d).bind(id).execute(pool).await?;
    }
    if let Some(s) = status {
        sqlx::query("UPDATE sys_dict SET status=?, updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_dict(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_dict WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn list_dict_items(pool: &SqlitePool, dict_id: i64) -> Result<Vec<DictItem>> {
    Ok(sqlx::query_as::<_, DictItem>(
        "SELECT * FROM sys_dict_item WHERE dict_id=? ORDER BY sort, id",
    )
    .bind(dict_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_dict_with_items(pool: &SqlitePool, id: i64) -> Result<Option<crate::dto::DictWithItems>> {
    let dict = get_dict(pool, id).await?;
    match dict {
        Some(d) => {
            let items = list_dict_items(pool, id).await?;
            Ok(Some(crate::dto::DictWithItems {
                id: d.id,
                name: d.name,
                code: d.code,
                description: d.description,
                status: d.status,
                items,
            }))
        }
        None => Ok(None),
    }
}

pub async fn create_dict_item(pool: &SqlitePool, dict_id: i64, label: &str, value: &str, sort: i32) -> Result<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_dict_item (dict_id, label, value, sort) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(dict_id)
    .bind(label)
    .bind(value)
    .bind(sort)
    .fetch_one(pool)
    .await?)
}

pub async fn update_dict_item(
    pool: &SqlitePool,
    id: i64,
    label: Option<&str>,
    value: Option<&str>,
    sort: Option<i32>,
    status: Option<&str>,
) -> Result<()> {
    if let Some(l) = label {
        sqlx::query("UPDATE sys_dict_item SET label=? WHERE id=?").bind(l).bind(id).execute(pool).await?;
    }
    if let Some(v) = value {
        sqlx::query("UPDATE sys_dict_item SET value=? WHERE id=?").bind(v).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_dict_item SET sort=? WHERE id=?").bind(s).bind(id).execute(pool).await?;
    }
    if let Some(st) = status {
        sqlx::query("UPDATE sys_dict_item SET status=?, updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(st).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_dict_item(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_dict_item WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}