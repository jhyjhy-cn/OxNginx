use log::LevelFilter;
use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub mod seed;
pub mod seed_menu;
pub mod seed_menu_json;
// ponytail: i18n 启动 seed 暂不使用，需要时取消下一行注释并启用 init.sql 建表。
// pub mod seed_i18n;

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

        sqlx::query(include_str!("sql/init.sql"))
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    /// 获取连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
