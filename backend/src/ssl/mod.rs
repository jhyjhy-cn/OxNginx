use tokio::process::Command;

/// 调用acme.sh安装证书到Nginx
#[allow(dead_code)]
pub async fn install_cert(
    acme_bin: &str,
    domain: &str,
) -> anyhow::Result<bool> {
    let output = Command::new(acme_bin)
        .args([
            "--install-cert",
            "-d",
            domain,
            "--key-file",
            &format!("/etc/nginx/ssl/{}.key", domain),
            "--fullchain-file",
            &format!("/etc/nginx/ssl/{}.pem", domain),
            "--reloadcmd",
            "systemctl reload nginx",
        ])
        .output()
        .await?;

    Ok(output.status.success())
}
