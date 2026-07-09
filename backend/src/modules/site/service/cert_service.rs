use crate::modules::site::dao::cert_dao;
use crate::modules::site::entity::certificate::Certificate;
use crate::AppState;

/// 获取所有证书
pub async fn get_all_certs(state: &AppState) -> anyhow::Result<Vec<Certificate>> {
    Ok(cert_dao::list_all_certs(state.db.pool()).await?)
}

/// 申请证书（调用 acme.sh）
pub async fn apply_cert(state: &AppState, domain: &str) -> anyhow::Result<Certificate> {
    let config = state.get_config();
    let acme_bin = &config.acme.bin;

    let output = crate::modules::common::util::cmd::silent_tokio_command(acme_bin)
        .args(["--issue", "-d", domain, "--standalone", "--force"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("证书申请失败: {}", stderr);
    }

    let cert_dir = format!("{}/{}_ecc", config.acme.home, domain);
    let cert_path = format!("{}/fullchain.cer", cert_dir);
    let key_path = format!("{}/{}.key", cert_dir, domain);

    if let Some(mut existing) = cert_dao::find_cert_by_domain(state.db.pool(), domain).await? {
        cert_dao::update_cert_paths(state.db.pool(), existing.id, &cert_path, &key_path).await?;
        existing.cert_path = Some(cert_path);
        existing.key_path = Some(key_path);
        return Ok(existing);
    }

    Ok(cert_dao::insert_cert_returning(state.db.pool(), domain, &cert_path, &key_path).await?)
}

/// 续期证书
pub async fn renew_cert(state: &AppState, id: i64) -> anyhow::Result<bool> {
    let cert = cert_dao::find_cert_by_id(state.db.pool(), id).await?;
    let cert = match cert {
        Some(c) => c,
        None => return Ok(false),
    };

    let config = state.get_config();
    let output = crate::modules::common::util::cmd::silent_tokio_command(&config.acme.bin)
        .args(["--renew", "-d", &cert.domain, "--force"])
        .output()
        .await?;

    Ok(output.status.success())
}