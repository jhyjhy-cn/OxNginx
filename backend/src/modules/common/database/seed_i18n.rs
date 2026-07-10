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
