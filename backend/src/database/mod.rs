use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
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
    pub async fn new(db_path: &str) -> anyhow::Result<Self> {
        // 确保目录存在
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_exists = Path::new(db_path).exists() && std::fs::metadata(db_path).map(|m| m.len() > 0).unwrap_or(false);

        let url = format!("sqlite:{}?mode=rwc", db_path);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
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
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                dept_id INTEGER,
                post_id INTEGER,
                disabled INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_sites (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                server_name TEXT NOT NULL,
                listen TEXT NOT NULL DEFAULT '80',
                ssl INTEGER NOT NULL DEFAULT 0,
                certificate_path TEXT,
                key_path TEXT,
                proxy_pass TEXT,
                root_path TEXT,
                config TEXT,
                remark TEXT,
                expire_time DATETIME,
                rewrite_rules TEXT,
                redirect_rules TEXT,
                hotlink_config TEXT,
                log_access_path TEXT,
                log_error_path TEXT,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_certificates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL UNIQUE,
                issuer TEXT,
                expire_time DATETIME,
                cert_path TEXT,
                key_path TEXT,
                auto_renew INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_backups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_id INTEGER,
                version INTEGER NOT NULL DEFAULT 1,
                config TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (site_id) REFERENCES sys_sites(id)
            );

            CREATE TABLE IF NOT EXISTS sys_upstreams (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                method TEXT NOT NULL DEFAULT 'round_robin',
                keepalive INTEGER DEFAULT 32,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_upstream_servers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                upstream_id INTEGER NOT NULL,
                address TEXT NOT NULL,
                weight INTEGER DEFAULT 1,
                max_fails INTEGER DEFAULT 3,
                fail_timeout TEXT DEFAULT '30s',
                backup INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'enabled',
                FOREIGN KEY (upstream_id) REFERENCES sys_upstreams(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sys_access_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_id INTEGER,
                rule_type TEXT NOT NULL,
                value TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (site_id) REFERENCES sys_sites(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sys_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                config TEXT NOT NULL,
                variables TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_file_notes (
                path TEXT PRIMARY KEY,
                note TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_reverse_proxies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                proxy_dir TEXT NOT NULL DEFAULT '/',
                target_url TEXT NOT NULL,
                cache INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (site_id) REFERENCES sys_sites(id) ON DELETE CASCADE
            );

            -- ===== RBAC =====
            CREATE TABLE IF NOT EXISTS sys_roles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                remark TEXT,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_depts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                parent_id INTEGER,
                name TEXT NOT NULL,
                sort INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_posts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                sort INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_menus (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                parent_id INTEGER,
                name TEXT NOT NULL,
                title TEXT NOT NULL,
                icon TEXT,
                path TEXT,
                component TEXT,
                type TEXT NOT NULL,
                permission TEXT,
                sort INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_user_roles (
                user_id INTEGER NOT NULL,
                role_id INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (user_id, role_id)
            );

            CREATE TABLE IF NOT EXISTS sys_role_menus (
                role_id INTEGER NOT NULL,
                menu_id INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (role_id, menu_id)
            );

            CREATE TABLE IF NOT EXISTS sys_i18n (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                locale TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(locale, key)
            );

            CREATE TABLE IF NOT EXISTS sys_dict (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                code TEXT NOT NULL UNIQUE,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_dict_item (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                dict_id INTEGER NOT NULL,
                label TEXT NOT NULL,
                value TEXT NOT NULL,
                sort INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (dict_id) REFERENCES sys_dict(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sys_tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token TEXT NOT NULL UNIQUE,
                user_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES sys_users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sys_operation_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                action TEXT NOT NULL,
                target TEXT,
                ip TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sys_login_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                ip TEXT,
                os TEXT,
                browser TEXT,
                user_agent TEXT,
                type TEXT NOT NULL DEFAULT 'login',
                status TEXT NOT NULL DEFAULT 'success',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 兼容旧库：sys_sites 表加 remark 列（已存在则忽略）
        let _ = sqlx::query("ALTER TABLE sys_sites ADD COLUMN remark TEXT")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sys_sites ADD COLUMN expire_time DATETIME")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sys_sites ADD COLUMN rewrite_rules TEXT")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sys_sites ADD COLUMN redirect_rules TEXT")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sys_sites ADD COLUMN hotlink_config TEXT")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sys_sites ADD COLUMN log_access_path TEXT")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sys_sites ADD COLUMN log_error_path TEXT")
            .execute(&self.pool)
            .await;

        // 兼容旧库：sys_users 加 RBAC 列
        let _ = sqlx::query("ALTER TABLE sys_users ADD COLUMN dept_id INTEGER")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sys_users ADD COLUMN post_id INTEGER")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sys_users ADD COLUMN disabled INTEGER NOT NULL DEFAULT 0")
            .execute(&self.pool)
            .await;

        // 兼容旧库：sys_menus 加 component 列
        let _ = sqlx::query("ALTER TABLE sys_menus ADD COLUMN component TEXT")
            .execute(&self.pool)
            .await;

        Ok(())
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
