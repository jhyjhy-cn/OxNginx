use log::LevelFilter;
use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub mod seed;
pub mod seed_i18n;
pub mod seed_menu;

/// 数据库封装
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 创建新的数据库连接
    pub async fn new(db_path: &str, log_sql: bool) -> anyhow::Result<Self> {
        // 确保目录存在
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_exists = Path::new(db_path).exists()
            && std::fs::metadata(db_path)
                .map(|m| m.len() > 0)
                .unwrap_or(false);

        let url = format!("sqlite:{}?mode=rwc", db_path);
        let opts = SqliteConnectOptions::from_url(&url.parse()?)?
            .log_statements(if log_sql { LevelFilter::Debug } else { LevelFilter::Off });
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;

        let db = Self { pool: pool.clone() };
        if db_exists {
            tracing::info!("  数据库已存在，跳过初始化");
        } else {
            tracing::info!("  创建数据表...");
            db.init_tables().await?;
            tracing::info!("  初始化种子数据...");
            seed::run(&pool).await?;
        }

        Ok(db)
    }

    /// 初始化数据库表
    async fn init_tables(&self) -> anyhow::Result<()> {
        // ponytail: sys_ 前缀；旧库无前缀则一次性 RENAME，平滑迁移
        for old in [
            "users",
            "sites",
            "certificates",
            "backups",
            "upstreams",
            "upstream_servers",
            "access_rules",
            "templates",
            "file_notes",
            "reverse_proxies",
        ] {
            let new = format!("sys_{}", old);
            let exists_new: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?",
            )
            .bind(&new)
            .fetch_one(&self.pool)
            .await?;
            if exists_new == 0 {
                let _ = sqlx::query(&format!("ALTER TABLE {} RENAME TO {}", old, new))
                    .execute(&self.pool)
                    .await;
            }
        }

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sys_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 用户ID
                username TEXT NOT NULL UNIQUE,          -- 用户名
                password TEXT NOT NULL,                 -- 密码（Argon2）
                nickname TEXT,                          -- 用户昵称
                phone TEXT,                             -- 手机号
                email TEXT,                             -- 邮箱
                gender TEXT,                            -- 性别：male/female/secret
                remark TEXT,                            -- 备注
                dept_id INTEGER,                        -- 部门ID
                post_id INTEGER,                        -- 岗位ID
                disabled INTEGER NOT NULL DEFAULT 0,    -- 是否禁用
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS site_sites (
                id INTEGER PRIMARY KEY AUTOINCREMENT,      -- 网站ID
                name TEXT NOT NULL,                       -- 网站名称
                server_name TEXT NOT NULL,                -- 域名
                listen TEXT NOT NULL DEFAULT '80',        -- 监听端口
                ssl INTEGER NOT NULL DEFAULT 0,           -- 是否启用SSL
                certificate_path TEXT,                    -- 证书路径
                key_path TEXT,                           -- 私钥路径
                proxy_pass TEXT,                          -- 反向代理地址
                root_path TEXT,                           -- 网站根目录
                config TEXT,                             -- Nginx配置
                remark TEXT,                             -- 备注
                expire_time DATETIME,                    -- 过期时间
                rewrite_rules TEXT,                      -- URL重写规则
                redirect_rules TEXT,                     -- 重定向规则
                hotlink_config TEXT,                     -- 防盗链配置
                log_access_path TEXT,                    -- 访问日志路径
                log_error_path TEXT,                     -- 错误日志路径
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS site_certificates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,   -- 证书ID
                domain TEXT NOT NULL UNIQUE,           -- 域名
                issuer TEXT,                           -- 颁发者
                expire_time DATETIME,                  -- 过期时间
                cert_path TEXT,                        -- 证书文件路径
                key_path TEXT,                         -- 私钥文件路径
                auto_renew INTEGER NOT NULL DEFAULT 1, -- 自动续期
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 创建时间
            );

            CREATE TABLE IF NOT EXISTS site_backups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,   -- 备份ID
                site_id INTEGER,                       -- 网站ID
                version INTEGER NOT NULL DEFAULT 1,    -- 版本号
                config TEXT NOT NULL,                  -- 配置内容
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                FOREIGN KEY (site_id) REFERENCES site_sites(id)
            );

            CREATE TABLE IF NOT EXISTS site_upstreams (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 负载均衡ID
                name TEXT NOT NULL UNIQUE,             -- 名称
                method TEXT NOT NULL DEFAULT 'round_robin', -- 负载均衡算法
                keepalive INTEGER DEFAULT 32,         -- 保持连接数
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS site_upstream_servers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 服务器ID
                upstream_id INTEGER NOT NULL,           -- 负载均衡ID
                address TEXT NOT NULL,                 -- 服务器地址
                weight INTEGER DEFAULT 1,               -- 权重
                max_fails INTEGER DEFAULT 3,            -- 最大失败次数
                fail_timeout TEXT DEFAULT '30s',        -- 失败超时
                backup INTEGER DEFAULT 0,               -- 备用服务器
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                FOREIGN KEY (upstream_id) REFERENCES site_upstreams(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS site_access_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 规则ID
                site_id INTEGER,                       -- 网站ID
                rule_type TEXT NOT NULL,               -- 规则类型
                value TEXT NOT NULL,                   -- 规则值
                description TEXT,                     -- 描述
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                FOREIGN KEY (site_id) REFERENCES site_sites(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS site_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 模板ID
                name TEXT NOT NULL UNIQUE,             -- 模板名称
                description TEXT,                     -- 描述
                config TEXT NOT NULL,                  -- 配置内容
                variables TEXT,                        -- 变量
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS file_notes (
                path TEXT PRIMARY KEY,                 -- 文件路径
                note TEXT NOT NULL,                    -- 备注内容
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 创建时间
            );

            CREATE TABLE IF NOT EXISTS site_reverse_proxies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 反向代理ID
                site_id INTEGER NOT NULL,              -- 网站ID
                name TEXT NOT NULL,                    -- 名称
                proxy_dir TEXT NOT NULL DEFAULT '/',   -- 代理路径
                target_url TEXT NOT NULL,              -- 目标URL
                cache INTEGER NOT NULL DEFAULT 0,      -- 是否启用缓存
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 更新时间
                FOREIGN KEY (site_id) REFERENCES site_sites(id) ON DELETE CASCADE
            );

            -- ===== RBAC =====
            CREATE TABLE IF NOT EXISTS sys_roles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 角色ID
                code TEXT NOT NULL UNIQUE,             -- 角色代码
                name TEXT NOT NULL,                    -- 角色名称
                remark TEXT,                           -- 备注
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS sys_depts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 部门ID
                parent_id INTEGER,                     -- 父部门ID
                name TEXT NOT NULL,                    -- 部门名称
                sort INTEGER NOT NULL DEFAULT 0,       -- 排序
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS sys_posts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 岗位ID
                code TEXT NOT NULL UNIQUE,             -- 岗位代码
                name TEXT NOT NULL,                    -- 岗位名称
                sort INTEGER NOT NULL DEFAULT 0,       -- 排序
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS sys_menus (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 菜单ID
                parent_id INTEGER,                     -- 父菜单ID
                name TEXT NOT NULL,                    -- 菜单名称
                title TEXT NOT NULL,                   -- 菜单标题
                icon TEXT,                             -- 图标
                path TEXT,                             -- 路由路径
                component TEXT,                        -- 组件路径
                type INTEGER NOT NULL,              -- 菜单类型：1=目录 2=菜单 3=按钮
                permission TEXT,                      -- 权限标识
                sort INTEGER NOT NULL DEFAULT 0,      -- 排序
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS sys_user_roles (
                user_id INTEGER NOT NULL,              -- 用户ID
                role_id INTEGER NOT NULL,              -- 角色ID
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                PRIMARY KEY (user_id, role_id)
            );

            CREATE TABLE IF NOT EXISTS sys_role_menus (
                role_id INTEGER NOT NULL,              -- 角色ID
                menu_id INTEGER NOT NULL,               -- 菜单ID
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                PRIMARY KEY (role_id, menu_id)
            );

            CREATE TABLE IF NOT EXISTS sys_i18n (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 国际化ID
                locale TEXT NOT NULL,                  -- 语言
                key TEXT NOT NULL,                     -- 键
                value TEXT NOT NULL,                   -- 值
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 更新时间
                UNIQUE(locale, key)
            );

            CREATE TABLE IF NOT EXISTS sys_dict (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 字典ID
                name TEXT NOT NULL,                     -- 字典名称
                code TEXT NOT NULL UNIQUE,              -- 字典代码
                description TEXT,                        -- 描述
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 更新时间
            );

            CREATE TABLE IF NOT EXISTS sys_dict_item (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 字典项ID
                dict_id INTEGER NOT NULL,              -- 字典ID
                label TEXT NOT NULL,                   -- 标签
                value TEXT NOT NULL,                   -- 值
                sort INTEGER NOT NULL DEFAULT 0,       -- 排序
                status INTEGER NOT NULL DEFAULT 1,  -- 状态：1=启用 0=禁用
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 更新时间
                FOREIGN KEY (dict_id) REFERENCES sys_dict(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sys_params (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 参数ID
                key TEXT NOT NULL UNIQUE,              -- 参数标识（英文唯一）
                value TEXT,                            -- 参数值
                name TEXT NOT NULL,                    -- 参数名（中文友好）
                group_code TEXT NOT NULL DEFAULT 'default', -- 分组
                remark TEXT,                           -- 备注
                sort INTEGER NOT NULL DEFAULT 0,       -- 排序
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_by INTEGER,
                updated_by INTEGER
            );

            CREATE TABLE IF NOT EXISTS sys_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 文件ID
                name TEXT NOT NULL UNIQUE,              -- 服务器文件名（uuid + 后缀）
                original_name TEXT NOT NULL,            -- 原始文件名
                suffix TEXT NOT NULL,                   -- 后缀（小写）
                size INTEGER NOT NULL DEFAULT 0,        -- 字节数
                mime_type TEXT,                         -- Content-Type
                md5 TEXT,                               -- 文件MD5
                path TEXT NOT NULL,                     -- 相对路径（不含域名）
                provider TEXT NOT NULL DEFAULT 'local', -- 服务商
                dept_id INTEGER,                        -- 部门ID
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_by INTEGER,
                updated_by INTEGER
            );

            CREATE TABLE IF NOT EXISTS sys_tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- TokenID
                token TEXT NOT NULL UNIQUE,            -- Token
                user_id INTEGER NOT NULL,               -- 用户ID
                username TEXT NOT NULL,                 -- 用户名
                expires_at DATETIME NOT NULL,           -- 过期时间
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- 创建时间
                FOREIGN KEY (user_id) REFERENCES sys_users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sys_operation_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 日志ID
                trace_id TEXT,                         -- 追踪ID
                username TEXT NOT NULL,                 -- 用户名
                module TEXT,                            -- 模块
                action TEXT NOT NULL,                   -- 操作
                method TEXT,                            -- 请求方法
                uri TEXT,                              -- 请求路径
                ip TEXT,                               -- IP地址
                status INTEGER NOT NULL DEFAULT 1, -- 状态
                cost_ms INTEGER,                       -- 耗时（毫秒）
                duration_ms INTEGER,                   -- 持续时间
                request_body TEXT,                      -- 请求体
                response_body TEXT,                     -- 响应体
                error_msg TEXT,                        -- 错误信息
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 创建时间
            );

            CREATE TABLE IF NOT EXISTS sys_login_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,  -- 日志ID
                username TEXT NOT NULL,                 -- 用户名
                ip TEXT,                               -- IP地址
                os TEXT,                               -- 操作系统
                browser TEXT,                           -- 浏览器
                user_agent TEXT,                       -- UserAgent
                type INTEGER NOT NULL DEFAULT 1, -- 日志类型
                status INTEGER NOT NULL DEFAULT 1, -- 状态
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP  -- 创建时间
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
