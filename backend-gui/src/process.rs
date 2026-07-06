use std::process::{Child, Command};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::AppHandle;

/// backend 进程管理
pub struct BackendProcess {
    child: Option<Child>,
}

impl BackendProcess {
    pub fn new() -> Self {
        Self { child: None }
    }

    /// 启动 backend 进程
    pub fn start(&mut self, _app_handle: &AppHandle) -> anyhow::Result<()> {
        // 开发模式：使用 backend/target/debug/ox-nginx.exe
        // 打包模式：使用 server/panel/ox-nginx.exe
        let backend_exe = if cfg!(debug_assertions) {
            // 开发模式 - exe 在 backend-gui/target/debug/，backend 在 backend/target/debug/
            let exe_dir = std::env::current_exe()?
                .parent()
                .unwrap()
                .to_path_buf(); // backend-gui/target/debug/

            let project_root = exe_dir
                .parent() // backend-gui/target/
                .and_then(|p| p.parent()) // backend-gui/
                .and_then(|p| p.parent()) // OxNginx/
                .unwrap();

            project_root.join("backend/target/debug/ox-nginx.exe")
        } else {
            // 打包模式 - ox-nginx.exe 在 server/panel/ 目录下
            let exe_dir = std::env::current_exe()?
                .parent()
                .unwrap()
                .to_path_buf(); // ox-nginx_1.1.0/

            exe_dir.join("server/panel/ox-nginx.exe")
        };

        if !backend_exe.exists() {
            anyhow::bail!("找不到 backend 可执行文件: {:?}", backend_exe);
        }

        self.start_process(&backend_exe)
    }

    /// 启动进程
    fn start_process(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        tracing::info!("启动 backend: {:?}", path);

        // 获取 exe所在目录，用于设置 CONFIG_PATH
        let exe_dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let config_path = exe_dir.join("configs").join("config.toml");

        let mut cmd = Command::new(path);
        cmd.arg("--headless");

        // 设置 CONFIG_PATH 环境变量
        if config_path.exists() {
            cmd.env("CONFIG_PATH", &config_path);
            tracing::info!("CONFIG_PATH: {:?}", config_path);
        }

        cmd.stdout(std::process::Stdio::null())
           .stderr(std::process::Stdio::null());

        let child = cmd.spawn()?;
        self.child = Some(child);
        tracing::info!("backend 进程已启动");
        Ok(())
    }

    /// 停止 backend 进程
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            tracing::info!("停止 backend 进程...");
            let _ = child.kill();
            let _ = child.wait();
            tracing::info!("backend 进程已停止");
        }
    }

    /// 检查进程是否运行
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// 获取后端可执行文件路径
    fn get_backend_exe(&self) -> anyhow::Result<std::path::PathBuf> {
        if cfg!(debug_assertions) {
            let exe_dir = std::env::current_exe()?
                .parent()
                .unwrap()
                .to_path_buf();

            let project_root = exe_dir
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .unwrap();

            Ok(project_root.join("backend/target/debug/ox-nginx.exe"))
        } else {
            // 打包模式 - ox-nginx.exe 在 server/panel/ 目录下
            let exe_dir = std::env::current_exe()?
                .parent()
                .unwrap()
                .to_path_buf();
            Ok(exe_dir.join("server/panel/ox-nginx.exe"))
        }
    }

    /// 手动启动 backend（用于命令调用）
    pub fn start_manual(&mut self) -> anyhow::Result<()> {
        let backend_exe = self.get_backend_exe()?;
        if !backend_exe.exists() {
            anyhow::bail!("找不到 backend 可执行文件: {:?}", backend_exe);
        }
        self.start_process(&backend_exe)
    }
}

/// 全局 backend 进程
static BACKEND_PROCESS: OnceLock<Mutex<BackendProcess>> = OnceLock::new();

/// 获取全局进程管理器
fn get_process() -> &'static Mutex<BackendProcess> {
    BACKEND_PROCESS.get_or_init(|| Mutex::new(BackendProcess::new()))
}

/// 启动 backend
pub fn start_backend(app_handle: &AppHandle, running: Arc<Mutex<bool>>) {
    let mut process = get_process().lock().unwrap();
    match process.start(app_handle) {
        Ok(_) => {
            *running.lock().unwrap() = true;
        }
        Err(e) => {
            tracing::error!("启动 backend 失败: {}", e);
        }
    }
}

/// 手动启动 backend（用于前端命令）
pub fn start_backend_manual() -> anyhow::Result<()> {
    let mut process = get_process().lock().unwrap();
    process.start_manual()
}

/// 停止 backend
pub fn stop_backend() {
    let mut process = get_process().lock().unwrap();
    process.stop();
}

/// 检查 backend 是否运行
pub fn is_backend_running() -> bool {
    let mut process = get_process().lock().unwrap();
    process.is_running()
}
