use serde::Serialize;

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
}

/// 获取系统信息
pub async fn get_system_info() -> anyhow::Result<SystemInfo> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let hostname = get_hostname().unwrap_or_else(|_| "unknown".to_string());
    let cpu_cores = num_cpus::get();

    // 获取 CPU 使用率
    let cpu_usage = get_cpu_usage().await.unwrap_or(0.0);

    // 获取内存信息
    let (memory_total, memory_used, memory_usage, swap_total, swap_used) = get_memory_info().await
        .unwrap_or((0, 0, 0.0, 0, 0));

    // 获取磁盘信息
    let (disk_total, disk_used, disk_usage) = get_disk_info().await
        .unwrap_or((0, 0, 0.0));

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
    })
}

/// 获取主机名
fn get_hostname() -> anyhow::Result<String> {
    let output = std::process::Command::new("hostname").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 获取 CPU 使用率（通过 /proc/stat）
async fn get_cpu_usage() -> anyhow::Result<f64> {
    // 读取两次 /proc/stat，计算差值
    let stat1 = tokio::fs::read_to_string("/proc/stat").await?;
    let cpu1 = parse_cpu_stat(&stat1)?;

    // 等待 100ms
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let stat2 = tokio::fs::read_to_string("/proc/stat").await?;
    let cpu2 = parse_cpu_stat(&stat2)?;

    let total1 = cpu1.0 + cpu1.1 + cpu1.2 + cpu1.3 + cpu1.4 + cpu1.5 + cpu1.6;
    let total2 = cpu2.0 + cpu2.1 + cpu2.2 + cpu2.3 + cpu2.4 + cpu2.5 + cpu2.6;

    let idle1 = cpu1.3;
    let idle2 = cpu2.3;

    let total_diff = total2 - total1;
    let idle_diff = idle2 - idle1;

    if total_diff == 0 {
        return Ok(0.0);
    }

    let usage = (1.0 - (idle_diff as f64 / total_diff as f64)) * 100.0;
    Ok(usage)
}

/// 解析 /proc/stat 中的 CPU 行
fn parse_cpu_stat(content: &str) -> anyhow::Result<(u64, u64, u64, u64, u64, u64, u64)> {
    let line = content.lines()
        .find(|l| l.starts_with("cpu "))
        .ok_or_else(|| anyhow::anyhow!("无法解析 /proc/stat"))?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 8 {
        return Err(anyhow::anyhow!("/proc/stat 格式错误"));
    }

    let user = parts[1].parse::<u64>()?;
    let nice = parts[2].parse::<u64>()?;
    let system = parts[3].parse::<u64>()?;
    let idle = parts[4].parse::<u64>()?;
    let iowait = parts[5].parse::<u64>()?;
    let irq = parts[6].parse::<u64>()?;
    let softirq = parts[7].parse::<u64>()?;

    Ok((user, nice, system, idle, iowait, irq, softirq))
}

/// 获取内存信息（通过 /proc/meminfo）
async fn get_memory_info() -> anyhow::Result<(u64, u64, f64, u64, u64)> {
    let content = tokio::fs::read_to_string("/proc/meminfo").await?;

    let mut mem_total = 0u64;
    let mut mem_free = 0u64;
    let mut mem_available = 0u64;
    let mut buffers = 0u64;
    let mut cached = 0u64;
    let mut swap_total = 0u64;
    let mut swap_free = 0u64;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let value = parts[1].parse::<u64>().unwrap_or(0);

        match parts[0] {
            "MemTotal:" => mem_total = value,
            "MemFree:" => mem_free = value,
            "MemAvailable:" => mem_available = value,
            "Buffers:" => buffers = value,
            "Cached:" => cached = value,
            "SwapTotal:" => swap_total = value,
            "SwapFree:" => swap_free = value,
            _ => {}
        }
    }

    // 如果有 MemAvailable，使用它；否则计算
    let available = if mem_available > 0 {
        mem_available
    } else {
        mem_free + buffers + cached
    };

    let mem_used = mem_total - available;
    let mem_usage = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let swap_used = swap_total - swap_free;

    // 转换为 MB
    let mem_total_mb = mem_total / 1024;
    let mem_used_mb = mem_used / 1024;
    let swap_total_mb = swap_total / 1024;
    let swap_used_mb = swap_used / 1024;

    Ok((mem_total_mb, mem_used_mb, mem_usage, swap_total_mb, swap_used_mb))
}

/// 获取磁盘信息
async fn get_disk_info() -> anyhow::Result<(u64, u64, f64)> {
    let output = tokio::process::Command::new("df")
        .args(["-B1", "/"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().nth(1).ok_or_else(|| anyhow::anyhow!("df 输出格式错误"))?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return Err(anyhow::anyhow!("df 输出格式错误"));
    }

    let total = parts[1].parse::<u64>()?;
    let used = parts[2].parse::<u64>()?;

    let total_gb = total / (1024 * 1024 * 1024);
    let used_gb = used / (1024 * 1024 * 1024);
    let usage = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok((total_gb, used_gb, usage))
}
