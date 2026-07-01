use crate::model::Certificate;
use crate::AppState;
use chrono::{NaiveDateTime, Utc};
use std::process::Command;

/// 从证书文件读取过期时间
pub async fn get_cert_expire_info(cert_path: &str) -> Option<NaiveDateTime> {
    #[cfg(target_os = "linux")]
    let output = Command::new("sudo")
        .arg("openssl")
        .args(["x509", "-in", cert_path, "-noout", "-enddate"])
        .output()
        .ok()?;
    #[cfg(target_os = "windows")]
    let output = Command::new("openssl")
        .args(["x509", "-in", cert_path, "-noout", "-enddate"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let output_str = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout); // e.g. "notAfter=Jul 15 12:00:00 2026 GMT"
    let date_str = output_str.trim().strip_prefix("notAfter=")?;
    // Parse "Jul 15 12:00:00 2026 GMT" -> NaiveDateTime
    let dt = chrono::DateTime::parse_from_str(&format!("{} +0000", date_str), "%b %d %H:%M:%S %Y %z")
        .ok()?;
    Some(dt.naive_utc())
}

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

    let config = state.get_config();
    let acme_bin = &config.acme.bin;

    // 调用acme.sh申请证书（Linux需要sudo提权才能绑定80端口）
    #[cfg(target_os = "linux")]
    let output = Command::new("sudo")
        .arg(acme_bin)
        .args(["--issue", "-d", domain, "--standalone", "--force"])
        .output()
        .await?;
    #[cfg(target_os = "windows")]
    let output = Command::new(acme_bin)
        .args(["--issue", "-d", domain, "--standalone", "--force"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("证书申请失败: {}", stderr);
    }

    // 获取证书路径（acme.sh 默认将 cert 保存在 _ecc 子目录）
    let cert_dir = format!("/root/.acme.sh/{}_ecc", domain);
    let cert_path = format!("{}/fullchain.cer", cert_dir);
    let key_path = format!("{}/{}.key", cert_dir, domain);

    // 先查是否已有该域名的证书记录，有则更新路径后返回
    if let Some(existing) = sqlx::query_as::<_, Certificate>(
        "SELECT * FROM certificates WHERE domain = ?",
    )
    .bind(domain)
    .fetch_optional(state.db.pool())
    .await?
    {
        // 更新为正确的 _ecc 路径
        sqlx::query("UPDATE certificates SET cert_path = ?, key_path = ? WHERE id = ?")
            .bind(&cert_path)
            .bind(&key_path)
            .bind(existing.id)
            .execute(state.db.pool())
            .await?;
        let mut updated = existing;
        updated.cert_path = Some(cert_path);
        updated.key_path = Some(key_path);
        return Ok(updated);
    }

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

    let config = state.get_config();
    #[cfg(target_os = "linux")]
    let output = Command::new("sudo")
        .arg(&config.acme.bin)
        .args(["--renew", "-d", &cert.domain, "--force"])
        .output()
        .await?;
    #[cfg(target_os = "windows")]
    let output = Command::new(&config.acme.bin)
        .args(["--renew", "-d", &cert.domain, "--force"])
        .output()
        .await?;

    Ok(output.status.success())
}
