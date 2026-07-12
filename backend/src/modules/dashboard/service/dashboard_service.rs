use crate::modules::common::dto::DashboardData;
use crate::AppState;

/// 获取Dashboard数据
pub async fn get_dashboard(state: &AppState) -> anyhow::Result<DashboardData> {
    // 获取站点数量
    let site_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM site_sites")
            .fetch_one(state.db.pool())
            .await?;

    // 获取证书数量
    let cert_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM site_certificates")
            .fetch_one(state.db.pool())
            .await?;

    // 获取Nginx信息
    let nginx_config = crate::modules::common::nginx::get_nginx_config(state).await?;
    let nginx_bin = nginx_config.bin.as_deref().unwrap_or("");
    let nginx_info = get_nginx_info(nginx_bin).await;

    // 获取系统信息
    let system_info = crate::modules::system::service::system_service::get_system_info(state).await
        .unwrap_or_else(|_| crate::modules::system::service::system_service::SystemInfo {
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
            app_memory: 0,
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
        app_memory: system_info.app_memory,
    })
}

struct NginxInfo {
    version: String,
    workers: u32,
    connections: u64,
}

/// 获取Nginx信息
async fn get_nginx_info(nginx_bin: &str) -> NginxInfo {
    use crate::modules::common::util::cmd;

    let output = cmd::silent_tokio_command(nginx_bin)
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
