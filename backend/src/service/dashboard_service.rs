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

    // 获取系统信息
    let system_info = super::system_service::get_system_info().await
        .unwrap_or_else(|_| super::system_service::SystemInfo {
            os: "unknown".to_string(),
            arch: "unknown".to_string(),
            hostname: "unknown".to_string(),
            cpu_cores: 0,
            cpu_usage: 0.0,
            memory_total: 0,
            memory_used: 0,
            memory_usage: 0.0,
            swap_total: 0,
            swap_used: 0,
            disk_total: 0,
            disk_used: 0,
            disk_usage: 0.0,
        });

    Ok(DashboardData {
        nginx_version: nginx_info.version,
        worker_count: nginx_info.workers,
        active_connections: nginx_info.connections,
        site_count,
        cert_count,
        cpu_usage: system_info.cpu_usage,
        memory_usage: system_info.memory_usage,
        memory_total: system_info.memory_total,
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
