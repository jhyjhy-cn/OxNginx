// ============================================================================
// 暂不使用: i18n 启动 seed (改用前端 ts 兜底,seed_i18n.rs 不再被 seed.rs 调用)
// 保留此处仅为方便后续恢复；恢复时取消下方块注释并启用 init.sql 的 sys_i18n 建表。
// ============================================================================
/*
use anyhow::Result;
use sqlx::SqlitePool;
use std::collections::BTreeMap;

const ZH_CN: &str = include_str!("language/zh-CN.json");
const EN_US: &str = include_str!("language/en-US.json");

/// 每次启动都跑：把 JSON 全量灌进 sys_i18n，INSERT OR IGNORE 保留管理员在线改的值
pub async fn seed_i18n(pool: &SqlitePool) -> Result<()> {
    let zh: BTreeMap<String, String> = serde_json::from_str(ZH_CN)?;
    let en: BTreeMap<String, String> = serde_json::from_str(EN_US)?;

    let mut tx = pool.begin().await?;
    for (locale, map) in [("zh-CN", &zh), ("en-US", &en)] {
        for (k, v) in map {
            sqlx::query("INSERT OR IGNORE INTO sys_i18n (locale, key, value) VALUES (?, ?, ?)")
                .bind(locale)
                .bind(k)
                .bind(v)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}
*/