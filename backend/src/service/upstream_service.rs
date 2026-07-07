use crate::dto::{CreateUpstreamRequest, UpdateUpstreamRequest};
use crate::model::{Upstream, UpstreamServer};
use crate::AppState;

/// 获取所有上游服务器
pub async fn get_all_upstreams(state: &AppState) -> anyhow::Result<Vec<Upstream>> {
    let upstreams = sqlx::query_as::<_, Upstream>(
        "SELECT * FROM sys_upstreams ORDER BY created_at DESC"
    )
    .fetch_all(state.db.pool())
    .await?;
    Ok(upstreams)
}

/// 获取单个上游服务器
pub async fn get_upstream(state: &AppState, id: i64) -> anyhow::Result<Option<Upstream>> {
    let upstream = sqlx::query_as::<_, Upstream>(
        "SELECT * FROM sys_upstreams WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await?;
    Ok(upstream)
}

/// 获取上游服务器及其节点
pub async fn get_upstream_with_servers(
    state: &AppState,
    id: i64,
) -> anyhow::Result<Option<(Upstream, Vec<UpstreamServer>)>> {
    let upstream = get_upstream(state, id).await?;
    if let Some(upstream) = upstream {
        let servers = sqlx::query_as::<_, UpstreamServer>(
            "SELECT * FROM sys_upstream_servers WHERE upstream_id = ? ORDER BY id"
        )
        .bind(id)
        .fetch_all(state.db.pool())
        .await?;
        Ok(Some((upstream, servers)))
    } else {
        Ok(None)
    }
}

/// 创建上游服务器
pub async fn create_upstream(
    state: &AppState,
    req: CreateUpstreamRequest,
) -> anyhow::Result<(Upstream, Vec<UpstreamServer>)> {
    // 创建上游服务器
    let upstream = sqlx::query_as::<_, Upstream>(
        r#"
        INSERT INTO sys_upstreams (name, method, keepalive)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&req.name)
    .bind(&req.method)
    .bind(req.keepalive)
    .fetch_one(state.db.pool())
    .await?;

    // 创建服务器节点
    let mut servers = Vec::new();
    for server_req in &req.servers {
        let server = sqlx::query_as::<_, UpstreamServer>(
            r#"
            INSERT INTO sys_upstream_servers (upstream_id, address, weight, max_fails, fail_timeout, backup)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(upstream.id)
        .bind(&server_req.address)
        .bind(server_req.weight)
        .bind(server_req.max_fails)
        .bind(&server_req.fail_timeout)
        .bind(if server_req.backup { 1 } else { 0 })
        .fetch_one(state.db.pool())
        .await?;
        servers.push(server);
    }

    Ok((upstream, servers))
}

/// 更新上游服务器
pub async fn update_upstream(
    state: &AppState,
    id: i64,
    req: UpdateUpstreamRequest,
) -> anyhow::Result<Option<(Upstream, Vec<UpstreamServer>)>> {
    let existing = get_upstream(state, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let name = req.name.unwrap_or(existing.name);
    let method = req.method.unwrap_or(existing.method);
    let keepalive = req.keepalive.unwrap_or(existing.keepalive);
    let status = req.status.unwrap_or(existing.status);

    // 更新上游服务器
    let upstream = sqlx::query_as::<_, Upstream>(
        r#"
        UPDATE sys_upstreams
        SET name = ?, method = ?, keepalive = ?, status = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(&name)
    .bind(&method)
    .bind(keepalive)
    .bind(&status)
    .bind(id)
    .fetch_optional(state.db.pool())
    .await?;

    if let Some(upstream) = upstream {
        // 如果提供了新的服务器列表，更新它
        if let Some(server_reqs) = req.servers {
            // 删除旧的服务器节点
            sqlx::query("DELETE FROM sys_upstream_servers WHERE upstream_id = ?")
                .bind(id)
                .execute(state.db.pool())
                .await?;

            // 创建新的服务器节点
            let mut servers = Vec::new();
            for server_req in &server_reqs {
                let server = sqlx::query_as::<_, UpstreamServer>(
                    r#"
                    INSERT INTO sys_upstream_servers (upstream_id, address, weight, max_fails, fail_timeout, backup)
                    VALUES (?, ?, ?, ?, ?, ?)
                    RETURNING *
                    "#,
                )
                .bind(id)
                .bind(&server_req.address)
                .bind(server_req.weight)
                .bind(server_req.max_fails)
                .bind(&server_req.fail_timeout)
                .bind(if server_req.backup { 1 } else { 0 })
                .fetch_one(state.db.pool())
                .await?;
                servers.push(server);
            }

            Ok(Some((upstream, servers)))
        } else {
            // 获取现有的服务器节点
            let servers = sqlx::query_as::<_, UpstreamServer>(
                "SELECT * FROM sys_upstream_servers WHERE upstream_id = ? ORDER BY id"
            )
            .bind(id)
            .fetch_all(state.db.pool())
            .await?;

            Ok(Some((upstream, servers)))
        }
    } else {
        Ok(None)
    }
}

/// 删除上游服务器
pub async fn delete_upstream(state: &AppState, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM sys_upstreams WHERE id = ?")
        .bind(id)
        .execute(state.db.pool())
        .await?;

    Ok(result.rows_affected() > 0)
}
