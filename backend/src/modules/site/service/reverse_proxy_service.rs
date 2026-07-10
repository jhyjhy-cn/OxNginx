use crate::modules::site::dao::reverse_proxy_dao;
use crate::modules::site::entity::reverse_proxy::ReverseProxy;
use crate::AppState;

/// 按站点列出反向代理
pub async fn list_by_site(state: &AppState, site_id: i64) -> anyhow::Result<Vec<ReverseProxy>> {
    Ok(reverse_proxy_dao::list_proxies_by_site(state.db.pool(), site_id).await?)
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
    Ok(reverse_proxy_dao::insert_proxy_returning(
        state.db.pool(),
        site_id,
        name,
        proxy_dir,
        target_url,
        cache,
    )
    .await?)
}

/// 更新反向代理
pub async fn update(
    state: &AppState,
    id: i64,
    name: Option<&str>,
    proxy_dir: Option<&str>,
    target_url: Option<&str>,
    cache: Option<i32>,
    status: Option<i32>,
) -> anyhow::Result<Option<ReverseProxy>> {
    let existing = reverse_proxy_dao::find_proxy_by_id(state.db.pool(), id).await?;
    let existing = match existing {
        Some(e) => e,
        None => return Ok(None),
    };

    let name = name.unwrap_or(&existing.name);
    let proxy_dir = proxy_dir.unwrap_or(&existing.proxy_dir);
    let target_url = target_url.unwrap_or(&existing.target_url);
    let cache = cache.unwrap_or(existing.cache);
    let status = status.unwrap_or(existing.status);

    Ok(reverse_proxy_dao::update_proxy_returning(
        state.db.pool(),
        id,
        name,
        proxy_dir,
        target_url,
        cache,
        status,
    )
    .await?)
}

/// 删除反向代理
pub async fn delete(state: &AppState, id: i64) -> anyhow::Result<bool> {
    Ok(reverse_proxy_dao::delete_proxy(state.db.pool(), id).await? > 0)
}