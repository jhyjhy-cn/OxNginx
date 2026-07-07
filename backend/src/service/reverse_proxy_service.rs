use crate::model::ReverseProxy;
use crate::AppState;

/// 按站点列出反向代理
pub async fn list_by_site(state: &AppState, site_id: i64) -> anyhow::Result<Vec<ReverseProxy>> {
    let proxies = sqlx::query_as::<_, ReverseProxy>(
        "SELECT * FROM sys_reverse_proxies WHERE site_id = ? ORDER BY id ASC",
    )
    .bind(site_id)
    .fetch_all(state.db.pool())
    .await?;
    Ok(proxies)
}

/// 创建反向代理
pub async fn create(
    state: &AppState,
    site_id: i64,
    name: &str,
    proxy_dir: &str,
    target_url: &str,
    cache: i32,
) -> anyhow::Result<ReverseProxy> {
    let proxy = sqlx::query_as::<_, ReverseProxy>(
        r#"
        INSERT INTO sys_reverse_proxies (site_id, name, proxy_dir, target_url, cache)
        VALUES (?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(site_id)
    .bind(name)
    .bind(proxy_dir)
    .bind(target_url)
    .bind(cache)
    .fetch_one(state.db.pool())
    .await?;
    Ok(proxy)
}

/// 更新反向代理
pub async fn update(
    state: &AppState,
    id: i64,
    name: Option<&str>,
    proxy_dir: Option<&str>,
    target_url: Option<&str>,
    cache: Option<i32>,
    status: Option<&str>,
) -> anyhow::Result<Option<ReverseProxy>> {
    let existing = sqlx::query_as::<_, ReverseProxy>("SELECT * FROM sys_reverse_proxies WHERE id = ?")
        .bind(id)
        .fetch_optional(state.db.pool())
        .await?;
    let existing = match existing {
        Some(e) => e,
        None => return Ok(None),
    };

    let name = name.unwrap_or(&existing.name);
    let proxy_dir = proxy_dir.unwrap_or(&existing.proxy_dir);
    let target_url = target_url.unwrap_or(&existing.target_url);
    let cache = cache.unwrap_or(existing.cache);
    let status = status.unwrap_or(&existing.status);

    let proxy = sqlx::query_as::<_, ReverseProxy>(
        r#"
        UPDATE sys_reverse_proxies
        SET name = ?, proxy_dir = ?, target_url = ?, cache = ?, status = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(proxy_dir)
    .bind(target_url)
    .bind(cache)
    .bind(status)
    .bind(id)
    .fetch_optional(state.db.pool())
    .await?;
    Ok(proxy)
}

/// 删除反向代理
pub async fn delete(state: &AppState, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM sys_reverse_proxies WHERE id = ?")
        .bind(id)
        .execute(state.db.pool())
        .await?;
    Ok(result.rows_affected() > 0)
}
