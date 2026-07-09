use crate::modules::common::util::cmd;
use chrono::NaiveDateTime;

/// 从证书文件读取过期时间
pub async fn get_cert_expire_info(cert_path: &str) -> Option<NaiveDateTime> {
    let output = cmd::silent_command("openssl")
        .args(["x509", "-in", cert_path, "-noout", "-enddate"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout); // e.g. "notAfter=Jul 15 12:00:00 2026 GMT"
    let date_str = output_str.trim().strip_prefix("notAfter=")?;
    let dt = chrono::DateTime::parse_from_str(&format!("{} +0000", date_str), "%b %d %H:%M:%S %Y %z")
        .ok()?;
    Some(dt.naive_utc())
}