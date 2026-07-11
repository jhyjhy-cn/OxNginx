use sqlx::{Row, Sqlite, SqlitePool, QueryBuilder};

use crate::modules::common::enums::LogStatus;
use crate::modules::sys::entity::menu::Menu;
use crate::modules::sys::entity::user::User;
use crate::modules::sys::service::user_service::UserListItem;

// ============== RBAC 相关查询 ==============

/// 列出所有启用的菜单权限码（admin 用）
pub async fn list_all_enabled_permissions(pool: &SqlitePool) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar(
        "SELECT permission FROM sys_menus WHERE permission IS NOT NULL AND status=?",
    )
    .bind(LogStatus::Success.as_i32())
    .fetch_all(pool)
    .await
}

/// 列出用户所有启用的角色编码
pub async fn list_enabled_role_codes_by_username(
    pool: &SqlitePool,
    username: &str,
) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar(
        "SELECT r.code FROM sys_roles r
         JOIN sys_user_roles ur ON r.id = ur.role_id
         JOIN sys_users u ON ur.user_id = u.id
         WHERE u.username = ? AND r.status=?",
    )
    .bind(username)
    .bind(LogStatus::Success.as_i32())
    .fetch_all(pool)
    .await
}

/// 列出用户所有启用的菜单权限码
pub async fn list_user_permissions(pool: &SqlitePool, username: &str) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar(
        "SELECT DISTINCT m.permission FROM sys_menus m
         JOIN sys_role_menus rm ON m.id = rm.menu_id
         JOIN sys_user_roles ur ON rm.role_id = ur.role_id
         JOIN sys_users u ON ur.user_id = u.id
         WHERE u.username = ? AND m.permission IS NOT NULL AND m.status=?",
    )
    .bind(username)
    .bind(LogStatus::Success.as_i32())
    .fetch_all(pool)
    .await
}

/// 列出所有启用的菜单（admin 用）
pub async fn list_all_enabled_menus(pool: &SqlitePool) -> sqlx::Result<Vec<Menu>> {
    sqlx::query_as::<_, Menu>(
        "SELECT id, parent_id, name, title, icon, path, component, type, permission, sort, status, created_at, updated_at
         FROM sys_menus WHERE status=? ORDER BY sort, id",
    )
    .bind(LogStatus::Success.as_i32())
    .fetch_all(pool)
    .await
}

/// 列出用户可见的菜单
pub async fn list_user_menus(pool: &SqlitePool, username: &str) -> sqlx::Result<Vec<Menu>> {
    sqlx::query_as::<_, Menu>(
        "SELECT DISTINCT m.id, m.parent_id, m.name, m.title, m.icon, m.path, m.component, m.type, m.permission, m.sort, m.status, m.created_at, m.updated_at
         FROM sys_menus m
         JOIN sys_role_menus rm ON m.id = rm.menu_id
         JOIN sys_user_roles ur ON rm.role_id = ur.role_id
         JOIN sys_users u ON ur.user_id = u.id
         WHERE u.username = ? AND m.status=?
         ORDER BY m.sort, m.id",
    )
    .bind(username)
    .bind(LogStatus::Success.as_i32())
    .fetch_all(pool)
    .await
}

// ============== 用户 CRUD ==============

/// 创建用户，返回 id
pub async fn insert_user_returning_id(
    pool: &SqlitePool,
    username: &str,
    hashed_password: &str,
) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_users (username, password) VALUES (?, ?) RETURNING id",
    )
    .bind(username)
    .bind(hashed_password)
    .fetch_one(pool)
    .await
}

/// 局部更新用户（每个字段独立 update）
#[allow(clippy::too_many_arguments)]
pub async fn update_user_fields(
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
) -> sqlx::Result<()> {
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
    Ok(())
}

/// 替换用户角色（先删后插）
pub async fn replace_user_roles(pool: &SqlitePool, user_id: i64, role_ids: &[i64]) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sys_user_roles WHERE user_id=?")
        .bind(user_id)
        .execute(pool)
        .await?;
    for rid in role_ids {
        sqlx::query("INSERT OR IGNORE INTO sys_user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(rid)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// 删除用户（保护 admin 不允许删除），返回受影响行数
pub async fn delete_user_protect_admin(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_users WHERE id=? AND username != 'admin'")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}

/// 重置密码
pub async fn update_user_password(pool: &SqlitePool, id: i64, hashed: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE sys_users SET password=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(hashed)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 统计 admin 用户（按 id）
pub async fn count_admin_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM sys_users WHERE id=? AND username='admin'")
        .bind(id)
        .fetch_one(pool)
        .await
}

/// 按 id 查用户
pub async fn find_user_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>("SELECT * FROM sys_users WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// 查用户角色 ID 列表
pub async fn list_user_role_ids(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar("SELECT role_id FROM sys_user_roles WHERE user_id=?")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

// ============== 用户分页（复杂查询）==============

/// 复杂分页计数：使用 QueryBuilder
pub async fn count_users_paged(
    pool: &SqlitePool,
    where_clause: &str,
    binds: &[String],
    int_binds: &[i64],
    dis_binds: &[i32],
) -> sqlx::Result<i64> {
    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT COUNT(DISTINCT u.id) FROM sys_users u \
         LEFT JOIN sys_depts d ON u.dept_id=d.id \
         LEFT JOIN sys_posts p ON u.post_id=p.id",
    );
    qb.push(where_clause);
    let mut query = qb.build_query_scalar::<i64>();
    for b in binds { query = query.bind(b); }
    for i in int_binds { query = query.bind(*i); }
    for d in dis_binds { query = query.bind(*d); }
    query.fetch_one(pool).await
}

/// 复杂分页列表：使用 QueryBuilder
#[allow(clippy::too_many_arguments)]
pub async fn list_users_paged(
    pool: &SqlitePool,
    where_clause: &str,
    binds: &[String],
    int_binds: &[i64],
    dis_binds: &[i32],
    page_size: i64,
    offset: i64,
) -> sqlx::Result<Vec<UserListItem>> {
    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT u.id, u.username, u.nickname, u.phone, u.email, u.gender, u.remark, \
                u.dept_id, d.name as dept_name, \
                u.post_id, p.name as post_name, \
                u.disabled, u.created_at \
         FROM sys_users u \
         LEFT JOIN sys_depts d ON u.dept_id=d.id \
         LEFT JOIN sys_posts p ON u.post_id=p.id",
    );
    qb.push(where_clause);
    qb.push(" ORDER BY u.id DESC LIMIT ?");
    qb.push(" OFFSET ?");
    let mut query = qb.build();
    for b in binds { query = query.bind(b); }
    for i in int_binds { query = query.bind(*i); }
    for d in dis_binds { query = query.bind(*d); }
    query = query.bind(page_size);
    query = query.bind(offset);
    let rows = query.fetch_all(pool).await?;
    Ok(rows
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
        .collect())
}