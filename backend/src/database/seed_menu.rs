use anyhow::Result;
use sqlx::SqlitePool;

/// 默认菜单种子（权威源）
/// 字段: (parent_name, name, title_i18n, icon, path, type, permission, component)
const DEFAULT_MENUS: &[(&str, &str, &str, &str, &str, &str, &str, &str)] = &[
    // 顶层菜单
    (
        "",
        "仪表盘",
        "menu.dashboard",
        "House",
        "/dashboard",
        "C",
        "sys:dashboard:view",
        "dashboard/index",
    ),
    (
        "",
        "站点管理",
        "menu.sites",
        "Connection",
        "/sites",
        "C",
        "sys:site:view",
        "sites/index",
    ),
    (
        "",
        "文件管理",
        "menu.files",
        "Folder",
        "/files",
        "C",
        "sys:file:view",
        "files/index",
    ),
    (
        "",
        "终端",
        "menu.terminal",
        "Monitor",
        "/terminal",
        "C",
        "sys:terminal:view",
        "terminal/index",
    ),
    (
        "",
        "设置",
        "menu.settings",
        "Setting",
        "/settings",
        "C",
        "sys:config:view",
        "settings/index",
    ),
    // 日志管理（目录）
    (
        "",
        "日志管理",
        "menu.logManagement",
        "DocumentCopy",
        "",
        "M",
        "",
        "",
    ),
    (
        "日志管理",
        "Nginx日志",
        "menu.nginxLogs",
        "DocumentCopy",
        "/logs/nginx",
        "C",
        "sys:log:view",
        "logs/index",
    ),
    (
        "日志管理",
        "操作日志",
        "menu.operationLogs",
        "Tickets",
        "/logs/operation",
        "C",
        "sys:log:view",
        "logs/operation",
    ),
    (
        "日志管理",
        "登录日志",
        "menu.loginLogs",
        "Promotion",
        "/logs/login",
        "C",
        "sys:log:view",
        "logs/login",
    ),
    // 按钮权限
    (
        "站点管理",
        "新增站点",
        "menu.siteAdd",
        "",
        "",
        "F",
        "sys:site:add",
        "",
    ),
    (
        "站点管理",
        "编辑站点",
        "menu.siteEdit",
        "",
        "",
        "F",
        "sys:site:edit",
        "",
    ),
    (
        "站点管理",
        "删除站点",
        "menu.siteDelete",
        "",
        "",
        "F",
        "sys:site:delete",
        "",
    ),
    // 权限管理
    (
        "",
        "权限管理",
        "menu.rbacManagement",
        "UserFilled",
        "",
        "M",
        "",
        "",
    ),
    (
        "权限管理",
        "用户管理",
        "menu.rbacUsers",
        "UserFilled",
        "/settings/rbac/users",
        "C",
        "sys:user:manage",
        "sys/users/index",
    ),
    (
        "权限管理",
        "角色管理",
        "menu.rbacRoles",
        "UserFilled",
        "/settings/rbac/roles",
        "C",
        "sys:role:manage",
        "sys/roles/index",
    ),
    (
        "权限管理",
        "部门管理",
        "menu.rbacDepts",
        "OfficeBuilding",
        "/settings/rbac/depts",
        "C",
        "sys:user:manage",
        "sys/depts/index",
    ),
    (
        "权限管理",
        "岗位管理",
        "menu.rbacPosts",
        "Postcard",
        "/settings/rbac/posts",
        "C",
        "sys:user:manage",
        "sys/posts/index",
    ),
    (
        "权限管理",
        "菜单管理",
        "menu.rbacMenus",
        "Menu",
        "/settings/rbac/menus",
        "C",
        "sys:menu:manage",
        "sys/menus/index",
    ),
    (
        "权限管理",
        "字典管理",
        "menu.rbacDicts",
        "Grid",
        "/settings/rbac/dicts",
        "C",
        "sys:config:view",
        "sys/dicts/index",
    ),
    (
        "权限管理",
        "国际化管理",
        "menu.rbacI18n",
        "MapLocation",
        "/settings/rbac/i18n",
        "C",
        "sys:config:view",
        "sys/i18n/index",
    ),
];

/// 启动种子：菜单 + super_admin 角色 + admin 绑定
pub async fn seed_menus(pool: &SqlitePool) -> Result<()> {
    dedup_menus(pool).await?;
    let seeded: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_menus WHERE parent_id IS NULL")
        .fetch_one(pool)
        .await?;
    if seeded == 0 {
        seed_default_menus(pool).await?;
    }
    let super_id = ensure_super_admin_role(pool).await?;
    bind_super_to_all_menus(pool, super_id).await?;
    bind_admin_user_to_super(pool).await?;
    Ok(())
}

/// 清理旧库重复 seed: 同 (parent_id, name) 保留最小 id
async fn dedup_menus(pool: &SqlitePool) -> Result<()> {
    let dup: Vec<(i64,)> = sqlx::query_as(
        "SELECT id FROM sys_menus WHERE id NOT IN (
            SELECT MIN(id) FROM sys_menus GROUP BY COALESCE(parent_id, -1), name
        )",
    )
    .fetch_all(pool)
    .await?;

    for (id,) in dup {
        let _ = sqlx::query("DELETE FROM sys_role_menus WHERE menu_id = ?")
            .bind(id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM sys_menus WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await;
    }
    Ok(())
}

/// 幂等 seed 菜单；二次启动零副作用
async fn seed_default_menus(pool: &SqlitePool) -> Result<()> {
    for (i, (parent_name, name, title, icon, path, mtype, perm, component)) in
        DEFAULT_MENUS.iter().enumerate()
    {
        let parent_id: Option<i64> = if parent_name.is_empty() {
            None
        } else {
            let top: Option<i64> =
                sqlx::query_scalar("SELECT id FROM sys_menus WHERE name = ? AND parent_id IS NULL")
                    .bind(parent_name)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten();
            match top {
                Some(id) => Some(id),
                None => sqlx::query_scalar("SELECT id FROM sys_menus WHERE name = ?")
                    .bind(parent_name)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten(),
            }
        };

        let sort = i as i32;
        sqlx::query(
            r#"INSERT OR IGNORE INTO sys_menus
               (parent_id, name, title, icon, path, type, permission, sort)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(parent_id)
        .bind(name)
        .bind(title)
        .bind(icon)
        .bind(path)
        .bind(mtype)
        .bind(perm)
        .bind(sort)
        .execute(pool)
        .await?;

        let _ = sqlx::query("UPDATE sys_menus SET sort = ?, component = ? WHERE name = ?")
            .bind(sort)
            .bind(if component.is_empty() {
                None
            } else {
                Some(*component)
            })
            .bind(name)
            .execute(pool)
            .await;
    }
    Ok(())
}

async fn ensure_super_admin_role(pool: &SqlitePool) -> Result<i64> {
    sqlx::query(
        "INSERT OR IGNORE INTO sys_roles (code, name, remark) VALUES ('super_admin', '超级管理员', '系统内置,所有权限')",
    )
    .execute(pool)
    .await?;

    Ok(
        sqlx::query_scalar("SELECT id FROM sys_roles WHERE code = 'super_admin'")
            .fetch_one(pool)
            .await?,
    )
}

async fn bind_super_to_all_menus(pool: &SqlitePool, role_id: i64) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO sys_role_menus (role_id, menu_id)
         SELECT ?, id FROM sys_menus",
    )
    .bind(role_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn bind_admin_user_to_super(pool: &SqlitePool) -> Result<()> {
    let admin_id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM sys_users WHERE username = 'admin'")
            .fetch_optional(pool)
            .await?;

    let Some(admin_id) = admin_id else {
        return Ok(());
    };
    let role_id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM sys_roles WHERE code = 'super_admin'")
            .fetch_optional(pool)
            .await?;

    let Some(role_id) = role_id else {
        return Ok(());
    };

    sqlx::query("INSERT OR IGNORE INTO sys_user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(admin_id)
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(())
}
