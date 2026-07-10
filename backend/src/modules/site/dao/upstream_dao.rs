use sqlx::SqlitePool;

use crate::modules::site::entity::upstream::{Upstream, UpstreamServer};

pub async fn list_all_upstreams(pool: &SqlitePool) -> sqlx::Result<Vec<Upstream>> {
    sqlx::query_as::<_, Upstream>("SELECT * FROM sys_upstreams ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn find_upstream_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Upstream>> {
    sqlx::query_as::<_, Upstream>("SELECT * FROM sys_upstreams WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_upstream_servers(
    pool: &SqlitePool,
    upstream_id: i64,
) -> sqlx::Result<Vec<UpstreamServer>> {
    sqlx::query_as::<_, UpstreamServer>(
        "SELECT * FROM sys_upstream_servers WHERE upstream_id = ? ORDER BY id",
    )
    .bind(upstream_id)
    .fetch_all(pool)
    .await
}

pub async fn insert_upstream_returning(
    pool: &SqlitePool,
    name: &str,
    method: &str,
    keepalive: i32,
) -> sqlx::Result<Upstream> {
    sqlx::query_as::<_, Upstream>(
        r#"
        INSERT INTO sys_upstreams (name, method, keepalive)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(method)
    .bind(keepalive)
    .fetch_one(pool)
    .await
}

pub async fn insert_upstream_server_returning(
    pool: &SqlitePool,
    upstream_id: i64,
    address: &str,
    weight: i32,
    max_fails: i32,
    fail_timeout: &str,
    backup: i32,
) -> sqlx::Result<UpstreamServer> {
    sqlx::query_as::<_, UpstreamServer>(
        r#"
        INSERT INTO sys_upstream_servers (upstream_id, address, weight, max_fails, fail_timeout, backup)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(upstream_id)
    .bind(address)
    .bind(weight)
    .bind(max_fails)
    .bind(fail_timeout)
    .bind(backup)
    .fetch_one(pool)
    .await
}

pub async fn update_upstream_returning(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    method: &str,
    keepalive: i32,
    status: i32,
) -> sqlx::Result<Option<Upstream>> {
    sqlx::query_as::<_, Upstream>(
        r#"
        UPDATE sys_upstreams
        SET name = ?, method = ?, keepalive = ?, status = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(method)
    .bind(keepalive)
    .bind(status)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_upstream_servers(pool: &SqlitePool, upstream_id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sys_upstream_servers WHERE upstream_id = ?")
        .bind(upstream_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_upstream(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_upstreams WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}