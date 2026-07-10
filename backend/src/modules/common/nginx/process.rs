use std::path::Path;

use crate::modules::common::dto::NginxTestResult;
use crate::modules::common::util::cmd;

/// 测试Nginx配置
pub async fn test_config(nginx_bin: &str) -> NginxTestResult {
    let nginx_dir = match get_nginx_dir(nginx_bin) {
        Some(dir) => dir,
        None => return NginxTestResult {
            success: false,
            message: "无法获取 nginx 安装目录".to_string(),
        },
    };
    let output = cmd::silent_tokio_command(nginx_bin)
        .current_dir(&nginx_dir)
        .arg("-t")
        .output()
        .await;
    match output {
        Ok(out) => NginxTestResult {
            success: out.status.success(),
            message: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => NginxTestResult {
            success: false,
            message: format!("执行nginx命令失败: {}", e),
        },
    }
}

/// Nginx 进程状态
#[derive(Debug, serde::Serialize)]
pub struct NginxStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub version: Option<String>,
    pub uptime: Option<String>,
    #[serde(default)]
    pub not_installed: bool,
}

/// 获取 nginx 可执行文件所在目录
fn get_nginx_dir(nginx_bin: &str) -> Option<std::path::PathBuf> {
    std::path::Path::new(nginx_bin).parent().map(|p| p.to_path_buf())
}

/// 获取 Nginx 运行状态
pub async fn get_nginx_status(nginx_bin: &str) -> NginxStatus {
    use std::env::consts::OS;

    // 先检测 nginx 是否可执行
    let version_check = cmd::silent_tokio_command(nginx_bin).arg("-v").output().await;
    let not_installed = version_check.is_err();

    if not_installed {
        return NginxStatus {
            running: false,
            pid: None,
            version: None,
            uptime: None,
            not_installed: true,
        };
    }

    let version = match &version_check {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            stderr.lines()
                .find(|l| l.contains("version"))
                .map(|l| l.trim().to_string())
        }
        _ => None,
    };

    let (pid, running) = if OS == "windows" {
        let output = cmd::silent_tokio_command("tasklist")
            .args(["/FI", "IMAGENAME eq nginx.exe", "/FO", "CSV", "/NH"])
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let pid = stdout.lines()
                    .find(|l| l.contains("nginx.exe"))
                    .and_then(|l| {
                        l.split(',').nth(1).and_then(|s| {
                            s.trim_matches('"').parse::<u32>().ok()
                        })
                    });
                (pid, pid.is_some())
            }
            _ => (None, false),
        }
    } else {
        let output = cmd::silent_tokio_command("pgrep")
            .args(["-x", "nginx"])
            .output()
            .await;

        let pid = match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.lines().next().and_then(|p| p.trim().parse::<u32>().ok())
            }
            _ => None,
        };
        (pid, pid.is_some())
    };

    let uptime = if let Some(pid) = pid {
        if OS == "windows" {
            let output = cmd::silent_tokio_command("wmic")
                .args(["process", "where", &format!("ProcessId={}", pid), "get", "CreationDate", "/value"])
                .output()
                .await;
            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    stdout.lines()
                        .find(|l| l.starts_with("CreationDate="))
                        .and_then(|l| {
                            let date_str = l.strip_prefix("CreationDate=")?.trim();
                            if date_str.len() >= 14 {
                                Some(format!("{}-{}-{} {}:{}:{}",
                                    &date_str[0..4], &date_str[4..6], &date_str[6..8],
                                    &date_str[8..10], &date_str[10..12], &date_str[12..14]))
                            } else {
                                None
                            }
                        })
                }
                _ => None,
            }
        } else {
            let output = cmd::silent_tokio_command("ps")
                .args(["-o", "etime=", "-p", &pid.to_string()])
                .output()
                .await;
            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    Some(stdout.trim().to_string())
                }
                _ => None,
            }
        }
    } else {
        None
    };

    NginxStatus {
        running,
        pid,
        version,
        uptime,
        not_installed: false,
    }
}

/// 启动 Nginx
pub async fn start_nginx(nginx_bin: &str, config_path: &str) -> anyhow::Result<bool> {
    let status = get_nginx_status(nginx_bin).await;
    tracing::info!("启动前状态: running={}, pid={:?}, not_installed={}", status.running, status.pid, status.not_installed);
    if status.running {
        tracing::info!("Nginx 已在运行 (PID {:?})，跳过启动", status.pid);
        return Ok(true);
    }

    let nginx_dir = get_nginx_dir(nginx_bin)
        .ok_or_else(|| anyhow::anyhow!("无法获取 nginx 安装目录"))?;
    tracing::info!("启动 Nginx: bin={}, conf={}", nginx_bin, config_path);

    let child = cmd::silent_tokio_command(nginx_bin)
        .current_dir(&nginx_dir)
        .args(["-c", config_path])
        .spawn();

    match child {
        Ok(_) => {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let status = get_nginx_status(nginx_bin).await;
            if status.running {
                tracing::info!("Nginx 启动成功 (PID {:?})", status.pid);
                Ok(true)
            } else {
                Err(anyhow::anyhow!("Nginx 启动后未检测到进程，请检查配置"))
            }
        }
        Err(e) => Err(anyhow::anyhow!("启动 Nginx 失败: {}", e)),
    }
}

/// 停止 Nginx
pub async fn stop_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    let nginx_dir = get_nginx_dir(nginx_bin)
        .ok_or_else(|| anyhow::anyhow!("无法获取 nginx 安装目录"))?;
    tracing::info!("停止 Nginx: bin={}", nginx_bin);
    let output = cmd::silent_tokio_command(nginx_bin)
        .current_dir(&nginx_dir)
        .args(["-s", "stop"])
        .output()
        .await?;
    Ok(output.status.success())
}

/// 重载 Nginx 配置
pub async fn reload_nginx(nginx_bin: &str) -> anyhow::Result<bool> {
    let nginx_dir = get_nginx_dir(nginx_bin)
        .ok_or_else(|| anyhow::anyhow!("无法获取 nginx 安装目录"))?;
    tracing::info!("重载 Nginx 配置: bin={}", nginx_bin);
    let output = cmd::silent_tokio_command(nginx_bin)
        .current_dir(&nginx_dir)
        .args(["-s", "reload"])
        .output()
        .await?;
    Ok(output.status.success())
}

/// 重启 Nginx
pub async fn restart_nginx(nginx_bin: &str, config_path: &str) -> anyhow::Result<bool> {
    stop_nginx(nginx_bin).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    start_nginx(nginx_bin, config_path).await
}

/// 把 zip 解压到 install_dir；zip 内若只有 nginx-{ver}/ 单层子目录，提到 install_dir 根。
pub fn extract_zip_to_install_dir(zip_path: &Path, install_dir: &Path, version: &str) -> anyhow::Result<()> {
    let top_subdir_name = format!("nginx-{}", version);

    let stale = install_dir.join(&top_subdir_name);
    let _ = std::fs::remove_dir_all(&stale);

    std::fs::create_dir_all(install_dir)?;

    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let out = install_dir.join(entry.mangled_name());
        if entry.is_dir() {
            std::fs::create_dir_all(&out)?;
        } else {
            if let Some(p) = out.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut out_file = std::fs::File::create(&out)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
    }

    let subdir = install_dir.join(&top_subdir_name);
    if subdir.is_dir() {
        for entry in std::fs::read_dir(&subdir)? {
            let entry = entry?;
            let dest = install_dir.join(entry.file_name());
            if !dest.exists() {
                std::fs::rename(entry.path(), &dest)?;
            } else {
                return Err(anyhow::anyhow!(
                    "目标已存在，无法合并: {} -> {}",
                    entry.path().display(),
                    dest.display()
                ));
            }
        }
        std::fs::remove_dir_all(&subdir)?;
    }
    Ok(())
}

/// Nginx 安装结果
#[derive(Debug)]
#[allow(dead_code)]
pub struct NginxInstallResult {
    pub bin: String,
    pub config: String,
    pub sites_enabled: String,
}

/// 一键安装 Nginx（Windows/Linux）
pub async fn install_nginx(install_dir: &str) -> anyhow::Result<NginxInstallResult> {
    use std::env::consts::OS;

    let os = OS;

    if os == "windows" {
        let nginx_version = "1.30.3";

        let zip_path = if cfg!(debug_assertions) {
            let libs_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("libs")
                .join("nginx")
                .join("windows");
            let local_zip = libs_dir.join(format!("nginx-{}.zip", nginx_version));
            if local_zip.exists() {
                tracing::info!("使用本地 Nginx 包: {}", local_zip.display());
                local_zip.to_string_lossy().to_string()
            } else {
                let dl_path = format!("{}\\{}.zip", install_dir, nginx_version);
                tracing::info!("下载 Nginx {}...", nginx_version);
                let output = cmd::silent_tokio_command("curl")
                    .args(["-L", "-o", &dl_path, &format!("https://nginx.org/download/nginx-{}.zip", nginx_version)])
                    .output()
                    .await?;
                if !output.status.success() {
                    return Err(anyhow::anyhow!("下载 Nginx 失败"));
                }
                dl_path
            }
        } else {
            let dl_path = format!("{}\\{}.zip", install_dir, nginx_version);
            tracing::info!("下载 Nginx {}...", nginx_version);
            let output = cmd::silent_tokio_command("curl")
                .args(["-L", "-o", &dl_path, &format!("https://nginx.org/download/nginx-{}.zip", nginx_version)])
                .output()
                .await?;
            if !output.status.success() {
                return Err(anyhow::anyhow!("下载 Nginx 失败"));
            }
            dl_path
        };

        tracing::info!("解压 Nginx...");

        tokio::fs::create_dir_all(install_dir).await?;

        let nginx_exe_path = Path::new(install_dir).join("nginx.exe");
        if nginx_exe_path.exists() {
            tracing::info!("Nginx 已存在，跳过解压: {}", nginx_exe_path.display());
        } else {
            let zip_path_pb = std::path::PathBuf::from(&zip_path);
            let install_dir_pb = std::path::PathBuf::from(install_dir);
            let version = nginx_version.to_string();
            tokio::task::spawn_blocking(move || {
                extract_zip_to_install_dir(&zip_path_pb, &install_dir_pb, &version)
            })
            .await??;
        }

        let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        if !Path::new(&zip_path).starts_with(project_root) {
            let _ = tokio::fs::remove_file(&zip_path).await;
        }

        let nginx_exe = format!("{}\\nginx.exe", install_dir);
        let nginx_conf = format!("{}\\conf\\nginx.conf", install_dir);
        let sites_enabled = format!("{}\\conf\\sites-enabled", install_dir);

        tracing::info!("Nginx 安装完成: {}", nginx_exe);
        Ok(NginxInstallResult {
            bin: nginx_exe,
            config: nginx_conf,
            sites_enabled,
        })
    } else {
        let src_tar = "/opt/oxnginx/server/nginx-src/nginx-1.30.3-linux-x86_64.tar.gz";

        if !std::path::Path::new(src_tar).exists() {
            return Err(anyhow::anyhow!(
                "nginx 预编译包不存在: {}，请确保部署时包含了 libs/nginx/linux/nginx-1.30.3-linux-x86_64.tar.gz",
                src_tar
            ));
        }

        let tmp_dir = "/tmp/oxnginx-nginx-extract";
        let _ = tokio::fs::remove_dir_all(tmp_dir).await;
        tokio::fs::create_dir_all(tmp_dir).await?;

        tracing::info!("解压 nginx 预编译包...");
        let output = cmd::silent_tokio_command("tar")
            .args(["-xzf", src_tar, "-C", tmp_dir])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = tokio::fs::remove_dir_all(tmp_dir).await;
            return Err(anyhow::anyhow!("解压 nginx 失败: {}", stderr));
        }

        let _ = tokio::fs::remove_dir_all(install_dir).await;
        tokio::fs::rename(format!("{}/nginx", tmp_dir), install_dir).await?;
        let _ = tokio::fs::remove_dir_all(tmp_dir).await;

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::symlink;
            let logs_dir = format!("{}/logs", install_dir);
            let _ = symlink("/opt/oxnginx/wwwlogs", &logs_dir);
        }

        let sites_enabled = format!("{}/conf/sites-enabled", install_dir);
        tokio::fs::create_dir_all(&sites_enabled).await?;

        let bin = format!("{}/sbin/nginx", install_dir);
        let nginx_conf = format!("{}/conf/nginx.conf", install_dir);
        tracing::info!("Nginx 安装完成: {}", bin);
        Ok(NginxInstallResult {
            bin,
            config: nginx_conf,
            sites_enabled,
        })
    }
}
