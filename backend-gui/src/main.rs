// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logging;
mod process;
mod tray;

use std::sync::{Arc, Mutex};
use serde::Serialize;

#[derive(Serialize)]
struct BackendStatus {
    running: bool,
    uptime: String,
    gui_memory: String,
    server_memory: String,
}

#[tauri::command]
fn get_backend_status() -> BackendStatus {
    let running = process::is_backend_running();

    BackendStatus {
        running,
        uptime: if running { "运行中" } else { "未运行" }.to_string(),
        gui_memory: process::get_gui_memory(),
        server_memory: if running { process::get_backend_memory() } else { "--".to_string() },
    }
}

#[tauri::command]
fn open_url(url: String) {
    let _ = open::that(&url);
}

#[tauri::command]
fn start_backend() -> Result<(), String> {
    process::start_backend_manual().map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_backend() {
    process::stop_backend();
}

fn main() {
    // 初始化日志
    logging::init();
    tracing::info!("OxNginx GUI 启动中...");

    // backend 进程状态
    let backend_running = Arc::new(Mutex::new(false));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![get_backend_status, start_backend, stop_backend, open_url])
        .setup(move |app| {
            // 创建系统托盘
            tray::create_tray(app.handle())?;

            // 启动 backend 进程
            let app_handle = app.handle().clone();
            let running = backend_running.clone();

            std::thread::spawn(move || {
                process::start_backend(&app_handle, running);
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // 窗口关闭时最小化到托盘，而不是退出
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
