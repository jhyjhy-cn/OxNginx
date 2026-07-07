use anyhow::Result;
use sqlx::SqlitePool;
use super::seed_i18n::seed_i18n;

/// 默认菜单种子（权威源）
/// 字段: (parent_name, name, title_i18n, icon, path, type, permission, component)
/// component: 前端视图文件名（views/ 下,无后缀）,用于动态路由
const DEFAULT_MENUS: &[(&str, &str, &str, &str, &str, &str, &str, &str)] = &[
    // 顶层菜单
    ("", "仪表盘",   "menu.dashboard", "House",       "/dashboard",  "C", "sys:dashboard:view", "Dashboard"),
    ("", "站点管理", "menu.sites",     "Connection",  "/sites",      "C", "sys:site:view",      "sites/index"),
    ("", "SSL证书",  "menu.ssl",       "Lock",        "/ssl",        "C", "sys:ssl:view",       "SSL"),
    ("", "配置模板", "menu.templates", "Document",    "/templates",  "C", "sys:template:view",  "Templates"),
    ("", "负载均衡", "menu.upstreams", "Share",       "/upstreams",  "C", "sys:upstream:view",  "Upstreams"),
    ("", "日志",     "menu.logs",      "DocumentCopy","/logs",       "C", "sys:log:view",       "Logs"),
    ("", "文件管理", "menu.files",     "Folder",      "/files",      "C", "sys:file:view",      "files/index"),
    ("", "终端",     "menu.terminal",  "Monitor",     "/terminal",   "C", "sys:terminal:view",  "Terminal"),
    ("", "设置",     "menu.settings",  "Setting",     "/settings",   "C", "sys:config:view",    "Settings"),

    // 按钮权限（挂在 站点管理 菜单下）
    ("站点管理", "新增站点",   "menu.siteAdd",    "", "", "F", "sys:site:add",    ""),
    ("站点管理", "编辑站点",   "menu.siteEdit",   "", "", "F", "sys:site:edit",   ""),
    ("站点管理", "删除站点",   "menu.siteDelete", "", "", "F", "sys:site:delete", ""),

    // SSL 按钮
    ("SSL证书", "申请证书",   "menu.sslApply",  "", "", "F", "sys:ssl:apply",  ""),
    ("SSL证书", "删除证书",   "menu.sslDelete", "", "", "F", "sys:ssl:delete", ""),

    // 模板按钮
    ("配置模板", "新增模板",   "menu.templateAdd",    "", "", "F", "sys:template:add",    ""),
    ("配置模板", "删除模板",   "menu.templateDelete", "", "", "F", "sys:template:delete", ""),

    // RBAC 管理（权限管理一级菜单下）
    ("",         "权限管理",   "menu.rbacManagement", "UserFilled", "",                 "M", "",                ""),
    ("权限管理", "用户管理",   "menu.rbacUsers",      "UserFilled",           "/settings/rbac/users", "C", "sys:user:manage", "RbacUsers"),
    ("权限管理", "角色管理",   "menu.rbacRoles",      "UserFilled",           "/settings/rbac/roles", "C", "sys:role:manage", "RbacRoles"),
    ("权限管理", "菜单管理",   "menu.rbacMenus",      "Menu",           "/settings/rbac/menus", "C", "sys:menu:manage", "RbacMenus"),
    ("权限管理", "部门管理",   "menu.rbacDepts",      "OfficeBuilding",  "/settings/rbac/depts", "C", "sys:user:manage", "RbacDepts"),
    ("权限管理", "岗位管理",   "menu.rbacPosts",      "Postcard",        "/settings/rbac/posts", "C", "sys:user:manage", "RbacPosts"),
    ("权限管理", "国际化管理", "menu.rbacI18n",       "MapLocation",       "/settings/rbac/i18n",  "C", "sys:config:view", "RbacI18n"),
    ("权限管理", "字典管理",   "menu.rbacDicts",       "Grid",              "/settings/rbac/dicts", "C", "sys:config:view", "RbacDicts"),
];

/// 启动种子：菜单 + super_admin 角色 + admin 用户绑定 + i18n
pub async fn run(pool: &SqlitePool) -> Result<()> {
    // ponytail: 幂等保护 + 清理历史重复 seed (id 段重复)
    dedup_menus(pool).await?;
    let seeded: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_menus WHERE parent_id IS NULL")
        .fetch_one(pool)
        .await?;
    if seeded > 0 {
        let super_id = ensure_super_admin_role(pool).await?;
        bind_super_to_all_menus(pool, super_id).await?;
        bind_admin_user_to_super(pool).await?;
    } else {
        seed_default_menus(pool).await?;
        let super_id = ensure_super_admin_role(pool).await?;
        bind_super_to_all_menus(pool, super_id).await?;
        bind_admin_user_to_super(pool).await?;
    }
    seed_i18n(pool).await?;
    Ok(())
}

/// 清理旧库重复 seed: 同 (parent_id, name, type) 保留最小 id
async fn dedup_menus(pool: &SqlitePool) -> Result<()> {
    // 删除重复项:保留每个 (parent_id, name) 组合里 id 最小的那条
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
    for (parent_name, name, title, icon, path, mtype, perm, component) in DEFAULT_MENUS {
        let parent_id: Option<i64> = if parent_name.is_empty() {
            None
        } else {
            // 先按顶级 (parent_id IS NULL) 找,找不到再按 name 任意找
            let top: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM sys_menus WHERE name = ? AND parent_id IS NULL",
            )
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

        sqlx::query(
            r#"INSERT OR IGNORE INTO sys_menus
               (parent_id, name, title, icon, path, type, permission, sort)
               VALUES (?, ?, ?, ?, ?, ?, ?, 0)"#,
        )
        .bind(parent_id)
        .bind(name)
        .bind(title)
        .bind(icon)
        .bind(path)
        .bind(mtype)
        .bind(perm)
        .execute(pool)
        .await?;

        // 回填 component 字段（已有行也覆盖）
        let _ = sqlx::query("UPDATE sys_menus SET component = ? WHERE name = ?")
            .bind(if component.is_empty() { None } else { Some(*component) })
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

    Ok(sqlx::query_scalar("SELECT id FROM sys_roles WHERE code = 'super_admin'")
        .fetch_one(pool)
        .await?)
}

/// super_admin 绑全部菜单
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

/// 已存在的 admin 用户自动绑定 super_admin 角色
async fn bind_admin_user_to_super(pool: &SqlitePool) -> Result<()> {
    let admin_id: Option<i64> = sqlx::query_scalar("SELECT id FROM sys_users WHERE username = 'admin'")
        .fetch_optional(pool)
        .await?;

    let Some(admin_id) = admin_id else { return Ok(()) };
    let role_id: Option<i64> = sqlx::query_scalar("SELECT id FROM sys_roles WHERE code = 'super_admin'")
        .fetch_optional(pool)
        .await?;

    let Some(role_id) = role_id else { return Ok(()) };

    sqlx::query("INSERT OR IGNORE INTO sys_user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(admin_id)
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(())
}