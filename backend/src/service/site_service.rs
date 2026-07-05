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

/// 证书信息（含剩余天数）
#[derive(Debug, serde::Serialize)]
pub struct SiteCertInfo {
    pub expire_time: Option<String>,
    pub days_remaining: Option<i64>,
}

impl Site {
    /// 读取证书过期时间
    pub async fn get_cert_expire_info(&self) -> Option<SiteCertInfo> {
        let cert_path = self.certificate_path.as_ref()?;
        if cert_path.is_empty() {
            return None;
        }
        let expire_time = crate::service::cert_service::get_cert_expire_info(cert_path).await?;
        let now = chrono::Utc::now().naive_utc();
        let days_remaining = (expire_time - now).num_days().max(0).try_into().ok();
        Some(SiteCertInfo {
            expire_time: Some(expire_time.format("%Y-%m-%d %H:%M:%S").to_string()),
            days_remaining,
        })
    }
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
        INSERT INTO sites (name, server_name, listen, ssl, certificate_path, key_path, proxy_pass, root_path, remark, expire_time, rewrite_rules, redirect_rules, hotlink_config, log_access_path, log_error_path)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .bind(&req.remark)
    .bind(&req.expire_time)
    .bind(&req.rewrite_rules)
    .bind(&req.redirect_rules)
    .bind(&req.hotlink_config)
    .bind(&req.log_access_path)
    .bind(&req.log_error_path)
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
    // Option<Option<String>>: None=未传, Some(None)=清空, Some(Some(v))=新值
    let certificate_path = req.certificate_path.unwrap_or(existing.certificate_path);
    let key_path = req.key_path.unwrap_or(existing.key_path);
    let proxy_pass = req.proxy_pass.unwrap_or(existing.proxy_pass);
    let root_path = req.root_path.unwrap_or(existing.root_path);
    let remark = req.remark.unwrap_or(existing.remark);
    let expire_time = req.expire_time.unwrap_or(existing.expire_time);
    let rewrite_rules = req.rewrite_rules.unwrap_or(existing.rewrite_rules);
    let redirect_rules = req.redirect_rules.unwrap_or(existing.redirect_rules);
    let hotlink_config = req.hotlink_config.unwrap_or(existing.hotlink_config);
    let log_access_path = req.log_access_path.unwrap_or(existing.log_access_path);
    let log_error_path = req.log_error_path.unwrap_or(existing.log_error_path);
    let status = req.status.unwrap_or(existing.status);

    let site = sqlx::query_as::<_, Site>(
        r#"
        UPDATE sites
        SET name = ?, server_name = ?, listen = ?, ssl = ?, certificate_path = ?, key_path = ?, proxy_pass = ?, root_path = ?, remark = ?, expire_time = ?, rewrite_rules = ?, redirect_rules = ?, hotlink_config = ?, log_access_path = ?, log_error_path = ?, status = ?, updated_at = CURRENT_TIMESTAMP
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
    .bind(&remark)
    .bind(&expire_time)
    .bind(&rewrite_rules)
    .bind(&redirect_rules)
    .bind(&hotlink_config)
    .bind(&log_access_path)
    .bind(&log_error_path)
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
