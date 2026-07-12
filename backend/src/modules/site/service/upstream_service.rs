use crate::modules::common::dto::{CreateUpstreamRequest, UpdateUpstreamRequest};
use crate::modules::site::dao::upstream_dao;
use crate::modules::site::entity::upstream::{Upstream, UpstreamServer};
use crate::AppState;

/// 获取所有上游服务器
pub async fn get_all_upstreams(state: &AppState) -> anyhow::Result<Vec<Upstream>> {
    Ok(upstream_dao::list_all_upstreams(state.db.pool()).await?)
}

/// 获取单个上游服务器
pub async fn get_upstream(state: &AppState, id: i64) -> anyhow::Result<Option<Upstream>> {
    Ok(upstream_dao::find_upstream_by_id(state.db.pool(), id).await?)
}

/// 获取上游服务器及其节点
pub async fn get_upstream_with_servers(
    state: &AppState,
    id: i64,
) -> anyhow::Result<Option<(Upstream, Vec<UpstreamServer>)>> {
    let upstream = get_upstream(state, id).await?;
    if let Some(upstream) = upstream {
        let servers = upstream_dao::list_upstream_servers(state.db.pool(), id).await?;
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
    let upstream = upstream_dao::insert_upstream_returning(
        state.db.pool(),
        &req.name,
        &req.method,
        req.keepalive,
    )
    .await?;

    let mut servers = Vec::new();
    for server_req in &req.servers {
        let server = upstream_dao::insert_upstream_server_returning(
            state.db.pool(),
            upstream.id,
            &server_req.address,
            server_req.weight,
            server_req.max_fails,
            &server_req.fail_timeout,
            if server_req.backup { 1 } else { 0 },
        )
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
    let existing = match get_upstream(state, id).await? {
        Some(e) => e,
        None => return Ok(None),
    };

    let name = req.name.unwrap_or(existing.name);
    let method = req.method.unwrap_or(existing.method);
    let keepalive = req.keepalive.unwrap_or(existing.keepalive);
    let status = req.status.unwrap_or(existing.status);

    let upstream = upstream_dao::update_upstream_returning(
        state.db.pool(),
        id,
        &name,
        &method,
        keepalive,
        status,
    )
    .await?;

    if let Some(upstream) = upstream {
        if let Some(server_reqs) = req.servers {
            upstream_dao::delete_upstream_servers(state.db.pool(), id).await?;
            let mut servers = Vec::new();
            for server_req in &server_reqs {
                let server = upstream_dao::insert_upstream_server_returning(
                    state.db.pool(),
                    id,
                    &server_req.address,
                    server_req.weight,
                    server_req.max_fails,
                    &server_req.fail_timeout,
                    if server_req.backup { 1 } else { 0 },
                )
                .await?;
                servers.push(server);
            }
            Ok(Some((upstream, servers)))
        } else {
            let servers = upstream_dao::list_upstream_servers(state.db.pool(), id).await?;
            Ok(Some((upstream, servers)))
        }
    } else {
        Ok(None)
    }
}

/// 删除上游服务器
pub async fn delete_upstream(state: &AppState, id: i64) -> anyhow::Result<bool> {
    Ok(upstream_dao::delete_upstream(state.db.pool(), id).await? > 0)
}