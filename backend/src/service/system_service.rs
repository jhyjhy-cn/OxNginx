use serde::Serialize;
use sysinfo::Disks;
// use tracing::debug;

use crate::AppState;

/// 系统信息
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub cpu_cores: usize,
    pub cpu_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_usage: f64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_usage: f64,
    pub app_memory: u64, // 本程序占用内存 MB
}

/// 获取系统信息（跨平台支持 Windows 和 Linux，复用 AppState 中的 System 实例避免重复初始化）
pub async fn get_system_info(state: &AppState) -> anyhow::Result<SystemInfo> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let hostname = get_hostname().unwrap_or_else(|_| "unknown".to_string());

    // 刷新进程列表，获取本程序内存占用
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu_all();
    sys.refresh_memory();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let app_memory = sys
        .process(state.pid)
        .map(|p| p.memory() / 1024 / 1024)
        .unwrap_or(0);

    let cpu_cores = sys.cpus().len();
    let cpu_usage = sys.global_cpu_usage() as f64;

    let memory_total = sys.total_memory() / 1024 / 1024;
    let memory_used = sys.used_memory() / 1024 / 1024;
    let memory_usage = if memory_total > 0 {
        (memory_used as f64 / memory_total as f64) * 100.0
    } else {
        0.0
    };

    let swap_total = sys.total_swap() / 1024 / 1024;
    let swap_used = sys.used_swap() / 1024 / 1024;

    // 磁盘信息
    let disks = Disks::new_with_refreshed_list();
    let (disk_total, disk_used) = disks
        .iter()
        .find(|d| {
            let mp = d.mount_point().as_os_str();
            mp == "/" || mp == "C:\\" || mp == "C:"
        })
        .map(|d| {
            let total = d.total_space() / 1024 / 1024 / 1024;
            let used = (d.total_space() - d.available_space()) / 1024 / 1024 / 1024;
            (total, used)
        })
        .unwrap_or((0, 0));
    let disk_usage = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    // debug!(
    //     "系统信息: os={}, arch={}, hostname={}, cpu_cores={}, cpu_usage={:.1}%, memory_usage={:.1}%, app_memory={}MB",
    //     os, arch, hostname, cpu_cores, cpu_usage, memory_usage, app_memory
    // );

    Ok(SystemInfo {
        os,
        arch,
        hostname,
        cpu_cores,
        cpu_usage,
        memory_total,
        memory_used,
        memory_usage,
        swap_total,
        swap_used,
        disk_total,
        disk_used,
        disk_usage,
        app_memory,
    })
}

/// 获取主机名
fn get_hostname() -> anyhow::Result<String> {
    let output = std::process::Command::new("hostname").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
