use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

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

        let url = format!("sqlite:{}?mode=rwc", db_path);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await?;

        let db = Self { pool };
        db.init_tables().await?;

        Ok(db)
    }

    /// 初始化数据库表
    async fn init_tables(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sites (
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
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS certificates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL UNIQUE,
                issuer TEXT,
                expire_time DATETIME,
                cert_path TEXT,
                key_path TEXT,
                auto_renew INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS backups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_id INTEGER,
                version INTEGER NOT NULL DEFAULT 1,
                config TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (site_id) REFERENCES sites(id)
            );

            CREATE TABLE IF NOT EXISTS upstreams (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                method TEXT NOT NULL DEFAULT 'round_robin',
                keepalive INTEGER DEFAULT 32,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS upstream_servers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                upstream_id INTEGER NOT NULL,
                address TEXT NOT NULL,
                weight INTEGER DEFAULT 1,
                max_fails INTEGER DEFAULT 3,
                fail_timeout TEXT DEFAULT '30s',
                backup INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'enabled',
                FOREIGN KEY (upstream_id) REFERENCES upstreams(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS access_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_id INTEGER,
                rule_type TEXT NOT NULL,
                value TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'enabled',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (site_id) REFERENCES sites(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                config TEXT NOT NULL,
                variables TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS file_notes (
                path TEXT PRIMARY KEY,
                note TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 兼容旧库：sites 表加 remark 列（已存在则忽略）
        let _ = sqlx::query("ALTER TABLE sites ADD COLUMN remark TEXT")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE sites ADD COLUMN expire_time DATETIME")
            .execute(&self.pool)
            .await;

        Ok(())
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
