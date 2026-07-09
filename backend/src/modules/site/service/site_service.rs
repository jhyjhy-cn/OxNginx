use crate::modules::common::dto::{CreateSiteRequest, UpdateSiteRequest};
use crate::modules::site::dao::site_dao;
use crate::modules::site::entity::site::Site;
use crate::AppState;

/// 获取所有站点
pub async fn get_all_sites(state: &AppState) -> anyhow::Result<Vec<Site>> {
    Ok(site_dao::list_all_sites(state.db.pool()).await?)
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
        let expire_time = crate::modules::common::util::cert_service::get_cert_expire_info(cert_path).await?;
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
    Ok(site_dao::find_site_by_id(state.db.pool(), id).await?)
}

/// 创建站点
pub async fn create_site(state: &AppState, req: CreateSiteRequest) -> anyhow::Result<Site> {
    let ssl_value = if req.ssl { 1 } else { 0 };
    Ok(site_dao::insert_site_returning(state.db.pool(), &req, ssl_value).await?)
}

/// 更新站点
pub async fn update_site(
    state: &AppState,
    id: i64,
    req: UpdateSiteRequest,
) -> anyhow::Result<Option<Site>> {
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

    Ok(site_dao::update_site_returning(
        state.db.pool(),
        id,
        &name,
        &server_name,
        &listen,
        ssl,
        certificate_path.as_ref(),
        key_path.as_ref(),
        proxy_pass.as_ref(),
        root_path.as_ref(),
        remark.as_ref(),
        expire_time.as_ref(),
        rewrite_rules.as_ref(),
        redirect_rules.as_ref(),
        hotlink_config.as_ref(),
        log_access_path.as_ref(),
        log_error_path.as_ref(),
        &status,
    )
    .await?)
}

/// 删除站点
pub async fn delete_site(state: &AppState, id: i64) -> anyhow::Result<bool> {
    Ok(site_dao::delete_site(state.db.pool(), id).await? > 0)
}