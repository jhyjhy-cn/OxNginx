use sqlx::SqlitePool;

use crate::modules::site::entity::reverse_proxy::ReverseProxy;

pub async fn list_proxies_by_site(pool: &SqlitePool, site_id: i64) -> sqlx::Result<Vec<ReverseProxy>> {
    sqlx::query_as::<_, ReverseProxy>(
        "SELECT * FROM site_reverse_proxies WHERE site_id = ? ORDER BY id ASC",
    )
    .bind(site_id)
    .fetch_all(pool)
    .await
}

pub async fn find_proxy_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<ReverseProxy>> {
    sqlx::query_as::<_, ReverseProxy>("SELECT * FROM site_reverse_proxies WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_proxy_returning(
    pool: &SqlitePool,
    site_id: i64,
    name: &str,
    proxy_dir: &str,
    target_url: &str,
    cache: i32,
) -> sqlx::Result<ReverseProxy> {
    sqlx::query_as::<_, ReverseProxy>(
        r#"
        INSERT INTO site_reverse_proxies (site_id, name, proxy_dir, target_url, cache)
        VALUES (?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(site_id)
    .bind(name)
    .bind(proxy_dir)
    .bind(target_url)
    .bind(cache)
    .fetch_one(pool)
    .await
}

pub async fn update_proxy_returning(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    proxy_dir: &str,
    target_url: &str,
    cache: i32,
    status: i32,
) -> sqlx::Result<Option<ReverseProxy>> {
    sqlx::query_as::<_, ReverseProxy>(
        r#"
        UPDATE site_reverse_proxies
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
    .fetch_optional(pool)
    .await
}

pub async fn delete_proxy(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM site_reverse_proxies WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}