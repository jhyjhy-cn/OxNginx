use crate::model::Certificate;
use crate::AppState;

/// 获取所有证书
pub async fn get_all_certs(state: &AppState) -> anyhow::Result<Vec<Certificate>> {
    let certs =
        sqlx::query_as::<_, Certificate>("SELECT * FROM certificates ORDER BY created_at DESC")
            .fetch_all(state.db.pool())
            .await?;
    Ok(certs)
}

/// 申请证书（调用acme.sh）
pub async fn apply_cert(
    state: &AppState,
    domain: &str,
) -> anyhow::Result<Certificate> {
    use tokio::process::Command;

    let acme_bin = &state.config.acme.bin;

    // 调用acme.sh申请证书
    let output = Command::new(acme_bin)
        .args(["--issue", "-d", domain, "--standalone"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("证书申请失败: {}", stderr);
    }

    // 获取证书路径
    let cert_dir = format!("/root/.acme.sh/{}", domain);
    let cert_path = format!("{}/fullchain.cer", cert_dir);
    let key_path = format!("{}/{}.key", cert_dir, domain);

    // 保存到数据库
    let cert = sqlx::query_as::<_, Certificate>(
        r#"
        INSERT INTO certificates (domain, issuer, cert_path, key_path, auto_renew)
        VALUES (?, 'Let''s Encrypt', ?, ?, 1)
        RETURNING *
        "#,
    )
    .bind(domain)
    .bind(&cert_path)
    .bind(&key_path)
    .fetch_one(state.db.pool())
    .await?;

    Ok(cert)
}

/// 续期证书
pub async fn renew_cert(state: &AppState, id: i64) -> anyhow::Result<bool> {
    use tokio::process::Command;

    let cert = sqlx::query_as::<_, Certificate>("SELECT * FROM certificates WHERE id = ?")
        .bind(id)
        .fetch_optional(state.db.pool())
        .await?;

    let cert = match cert {
        Some(c) => c,
        None => return Ok(false),
    };

    let output = Command::new(&state.config.acme.bin)
        .args(["--renew", "-d", &cert.domain, "--force"])
        .output()
        .await?;

    Ok(output.status.success())
}
