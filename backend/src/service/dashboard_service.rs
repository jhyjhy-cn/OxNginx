use crate::dto::DashboardData;
use crate::AppState;

/// 获取Dashboard数据
pub async fn get_dashboard(state: &AppState) -> anyhow::Result<DashboardData> {
    // 获取站点数量
    let site_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sites")
            .fetch_one(state.db.pool())
            .await?;

    // 获取证书数量
    let cert_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM certificates")
            .fetch_one(state.db.pool())
            .await?;

    // 获取Nginx信息
    let nginx_info = get_nginx_info(&state.config.nginx.bin).await;

    Ok(DashboardData {
        nginx_version: nginx_info.version,
        worker_count: nginx_info.workers,
        active_connections: nginx_info.connections,
        site_count,
        cert_count,
        cpu_usage: 0.0,
        memory_usage: 0.0,
        memory_total: 0,
    })
}

struct NginxInfo {
    version: String,
    workers: u32,
    connections: u64,
}

/// 获取Nginx信息
async fn get_nginx_info(nginx_bin: &str) -> NginxInfo {
    use tokio::process::Command;

    let output = Command::new(nginx_bin)
        .arg("-v")
        .output()
        .await;

    let version = match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            stderr
                .lines()
                .find(|l| l.contains("version"))
                .map(|l| l.to_string())
                .unwrap_or_else(|| "unknown".into())
        }
        Err(_) => "not installed".into(),
    };

    NginxInfo {
        version,
        workers: 0,
        connections: 0,
    }
}
