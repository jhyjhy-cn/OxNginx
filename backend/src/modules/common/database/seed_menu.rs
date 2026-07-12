use anyhow::Result;
use sqlx::SqlitePool;

use super::seed_menu_json::load_default_menus;

/// 默认菜单种子（权威源）
/// 数据落在 `seed/menu.json`，结构见 [`super::seed_menu_json::MenuRow`]

/// 启动种子：菜单 + super_admin 角色 + 默认部门/岗位
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
    seed_default_roles(pool).await?;
    seed_default_dept(pool).await?;
    seed_default_post(pool).await?;
    seed_default_dicts(pool).await?;
    crate::modules::sys::service::param_service::ensure_default_params(pool).await?;
    crate::modules::sys::service::param_service::ensure_nginx_params(pool).await?;
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
    for (i, m) in load_default_menus().iter().enumerate() {
        let parent_id: Option<i64> = if m.p.is_empty() {
            None
        } else {
            let top: Option<i64> =
                sqlx::query_scalar("SELECT id FROM sys_menus WHERE name = ? AND parent_id IS NULL")
                    .bind(&m.p)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten();
            match top {
                Some(id) => Some(id),
                None => sqlx::query_scalar("SELECT id FROM sys_menus WHERE name = ?")
                    .bind(&m.p)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten(),
            }
        };

        let sort = i as i32;
        let type_int: i32 = match m.y.as_str() {
            "M" => 1,
            "C" => 2,
            "F" => 3,
            _ => 0,
        };
        sqlx::query(
            r#"INSERT OR IGNORE INTO sys_menus
               (parent_id, name, title, icon, path, type, permission, sort)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(parent_id)
        .bind(&m.n)
        .bind(&m.t)
        .bind(&m.i)
        .bind(&m.u)
        .bind(type_int)
        .bind(&m.pm)
        .bind(sort)
        .execute(pool)
        .await?;

        let _ = sqlx::query("UPDATE sys_menus SET sort = ?, component = ? WHERE name = ?")
            .bind(sort)
            .bind(if m.c.is_empty() {
                None
            } else {
                Some(m.c.as_str())
            })
            .bind(&m.n)
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

/// 默认角色（幂等，code 唯一）；不绑菜单权限，由管理员后续分配
async fn seed_default_roles(pool: &SqlitePool) -> Result<()> {
    // (code, name)
    const ROLES: &[(&str, &str)] = &[
        ("admin", "管理员"),
        ("chairman", "董事长"),
        ("general_manager", "总经理"),
    ];
    for (code, name) in ROLES {
        sqlx::query("INSERT OR IGNORE INTO sys_roles (code, name) VALUES (?, ?)")
            .bind(code)
            .bind(name)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// 默认数据字典（幂等，按 code 判存；已有字典不动，仅追加新项）
async fn seed_default_dicts(pool: &SqlitePool) -> Result<()> {
    // (dict_code, dict_name, dict_description, items: [(label, value)])；排序由数组顺序决定
    const DICTS: &[(&str, &str, &str, &[(&str, i32)])] = &[
        (
            "common_status",
            "通用状态",
            "全局启用/禁用状态：1=启用 0=禁用",
            &[("启用", 1), ("禁用", 0)],
        ),
        (
            "log_status",
            "操作状态",
            "操作日志结果：1=成功 0=失败",
            &[("成功", 1), ("失败", 0)],
        ),
        (
            "login_log_type",
            "登录日志类型",
            "登录/退出登录",
            &[("登录", 1), ("退出", 0)],
        ),
        (
            "menu_type",
            "菜单类型",
            "目录/菜单/按钮权限",
            &[("目录", 1), ("菜单", 2), ("按钮", 3)],
        ),
        (
            "upstream_method",
            "负载均衡算法",
            "Nginx upstream 负载均衡方式",
            &[
                ("轮询", 0),
                ("IP Hash", 1),
                ("最少连接", 2),
                ("URL Hash", 3),
            ],
        ),
        (
            "operation_module",
            "操作日志模块",
            "操作日志归属模块",
            &[
                ("站点管理", 0),
                ("权限管理", 1),
                ("Nginx", 2),
                ("文件管理", 3),
                ("配置管理", 4),
                ("访问控制", 5),
                ("备份管理", 6),
                ("模板管理", 7),
                ("上游服务", 8),
                ("反向代理", 9),
                ("系统设置", 10),
            ],
        ),
    ];

    for (code, name, remark, items) in DICTS {
        // 幂等：已有字典不动
        let exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sys_dict WHERE code=?")
                .bind(code)
                .fetch_one(pool)
                .await?;
        if exists == 0 {
            sqlx::query("INSERT INTO sys_dict (code, name, remark) VALUES (?, ?, ?)")
                .bind(code)
                .bind(name)
                .bind(remark)
                .execute(pool)
                .await?;
        }

        // 取 dict_id
        let dict_id: i64 =
            sqlx::query_scalar("SELECT id FROM sys_dict WHERE code=?")
                .bind(code)
                .fetch_one(pool)
                .await?;

        // 幂等：按 (dict_id, value) 判重；排序由数组顺序决定
        for (sort, (label, value)) in items.iter().enumerate() {
            let item_exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sys_dict_item WHERE dict_id=? AND value=?",
            )
            .bind(dict_id)
            .bind(value)
            .fetch_one(pool)
            .await?;
            if item_exists == 0 {
                sqlx::query(
                    "INSERT INTO sys_dict_item (dict_id, label, value, sort) VALUES (?, ?, ?, ?)",
                )
                .bind(dict_id)
                .bind(label)
                .bind(value)
                .bind(sort as i32)
                .execute(pool)
                .await?;
            }
        }
    }
    Ok(())
}

/// 创建默认部门（id=1 根公司 + 两个分公司子级）
async fn seed_default_dept(pool: &SqlitePool) -> Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_depts WHERE id=1")
        .fetch_one(pool)
        .await?;
    if count == 0 {
        sqlx::query("INSERT INTO sys_depts (id, name, sort) VALUES (1, 'xx有限公司', 0)")
            .execute(pool)
            .await?;
    } else {
        // 更新名称
        let _ = sqlx::query("UPDATE sys_depts SET name='xx有限公司' WHERE id=1")
            .execute(pool)
            .await;
    }

    // 两个分公司子级（幂等：按 name+parent_id 判存）
    for (i, name) in ["上海分公司", "徐州分公司"].iter().enumerate() {
        let exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sys_depts WHERE parent_id=1 AND name=?")
                .bind(name)
                .fetch_one(pool)
                .await?;
        if exists == 0 {
            sqlx::query("INSERT INTO sys_depts (parent_id, name, sort) VALUES (1, ?, ?)")
                .bind(name)
                .bind(i as i32)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

/// 创建默认岗位（id=1 默认岗 + 客服/测试/研发岗）
async fn seed_default_post(pool: &SqlitePool) -> Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_posts WHERE id=1")
        .fetch_one(pool)
        .await?;
    if count == 0 {
        sqlx::query("INSERT INTO sys_posts (id, code, name, sort) VALUES (1, 'default', '默认岗位', 0)")
            .execute(pool)
            .await?;
    } else {
        // 更新名称
        let _ = sqlx::query("UPDATE sys_posts SET name='默认岗位' WHERE id=1")
            .execute(pool)
            .await;
    }

    // 业务岗位（幂等，code 唯一）
    const POSTS: &[(&str, &str)] = &[
        ("service", "客服岗"),
        ("qa", "测试岗"),
        ("dev", "研发岗"),
    ];
    for (i, (code, name)) in POSTS.iter().enumerate() {
        sqlx::query("INSERT OR IGNORE INTO sys_posts (code, name, sort) VALUES (?, ?, ?)")
            .bind(code)
            .bind(name)
            .bind(i as i32 + 1)
            .execute(pool)
            .await?;
    }
    Ok(())
}
