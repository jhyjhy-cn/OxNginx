use crate::dto::{CreateSiteRequest, UpdateSiteRequest};
use crate::model::Site;
use crate::AppState;

/// 获取所有站点
pub async fn get_all_sites(state: &AppState) -> anyhow::Result<Vec<Site>> {
    let sites = sqlx::query_as::<_, Site>("SELECT * FROM sites ORDER BY created_at DESC")
        .fetch_all(state.db.pool())
        .await?;
    Ok(sites)
}

/// 获取单个站点
pub async fn get_site(state: &AppState, id: i64) -> anyhow::Result<Option<Site>> {
    let site = sqlx::query_as::<_, Site>("SELECT * FROM sites WHERE id = ?")
        .bind(id)
        .fetch_optional(state.db.pool())
        .await?;
    Ok(site)
}

/// 创建站点
pub async fn create_site(state: &AppState, req: CreateSiteRequest) -> anyhow::Result<Site> {
    let ssl_value = if req.ssl { 1 } else { 0 };
    let result = sqlx::query_as::<_, Site>(
        r#"
        INSERT INTO sites (name, server_name, listen, ssl, certificate_path, key_path, proxy_pass, root_path)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&req.name)
    .bind(&req.server_name)
    .bind(&req.listen)
    .bind(ssl_value)
    .bind(&req.certificate_path)
    .bind(&req.key_path)
    .bind(&req.proxy_pass)
    .bind(&req.root_path)
    .fetch_one(state.db.pool())
    .await?;

    Ok(result)
}

/// 更新站点
pub async fn update_site(
    state: &AppState,
    id: i64,
    req: UpdateSiteRequest,
) -> anyhow::Result<Option<Site>> {
    // 先获取现有站点
    let existing = get_site(state, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let name = req.name.unwrap_or(existing.name);
    let server_name = req.server_name.unwrap_or(existing.server_name);
    let listen = req.listen.unwrap_or(existing.listen);
    let ssl = req
        .ssl
        .map(|v| if v { 1 } else { 0 })
        .unwrap_or(existing.ssl);
    let certificate_path = req
        .certificate_path
        .or(existing.certificate_path);
    let key_path = req.key_path.or(existing.key_path);
    let proxy_pass = req.proxy_pass.or(existing.proxy_pass);
    let root_path = req.root_path.or(existing.root_path);
    let status = req.status.unwrap_or(existing.status);

    let site = sqlx::query_as::<_, Site>(
        r#"
        UPDATE sites
        SET name = ?, server_name = ?, listen = ?, ssl = ?, certificate_path = ?, key_path = ?, proxy_pass = ?, root_path = ?, status = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(&name)
    .bind(&server_name)
    .bind(&listen)
    .bind(ssl)
    .bind(&certificate_path)
    .bind(&key_path)
    .bind(&proxy_pass)
    .bind(&root_path)
    .bind(&status)
    .bind(id)
    .fetch_optional(state.db.pool())
    .await?;

    Ok(site)
}

/// 删除站点
pub async fn delete_site(state: &AppState, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM sites WHERE id = ?")
        .bind(id)
        .execute(state.db.pool())
        .await?;

    Ok(result.rows_affected() > 0)
}
