use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::Deserialize;

use crate::auth;
use crate::AppState;

/// 找到 \x1b[2J（清屏）序列之后的位置，用于过滤 ConPTY 初始清屏
fn find_after_clear_screen(data: &[u8]) -> Option<usize> {
    // 匹配 \x1b[2J（CSI 2 J = erase display）
    for i in 0..data.len().saturating_sub(3) {
        if data[i] == 0x1b && data[i + 1] == b'[' && data[i + 2] == b'2' && data[i + 3] == b'J'
        {
            return Some(i + 4);
        }
    }
    None
}

#[derive(Deserialize)]
pub struct TerminalQuery {
    token: String,
    cols: Option<u16>,
    rows: Option<u16>,
    shell: Option<String>,
}

pub async fn terminal_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<TerminalQuery>,
) -> impl IntoResponse {
    let config = state.get_config();
    if auth::verify_token(&query.token, &config.auth.jwt_secret).is_err() {
        return axum::http::Response::builder()
            .status(401)
            .body("Unauthorized".into())
            .unwrap();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, query.cols, query.rows, query.shell))
}

async fn handle_socket(socket: WebSocket, cols: Option<u16>, rows: Option<u16>, shell: Option<String>) {
    let cols = cols.unwrap_or(80);
    let rows = rows.unwrap_or(24);
    let shell = shell.unwrap_or_else(|| "powershell".to_string());
    tracing::info!("终端 WebSocket 已连接, cols={}, rows={}, shell={}", cols, rows, shell);

    let pty_system = NativePtySystem::default();
    let pty = match pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(pty) => pty,
        Err(e) => {
            tracing::error!("创建 PTY 失败: {}", e);
            return;
        }
    };

    #[cfg(target_os = "windows")]
    let mut cmd = match shell.as_str() {
        "cmd" => {
            let mut c = CommandBuilder::new("cmd.exe");
            c.args(["/K", "chcp 65001 >nul"]);
            c
        }
        _ => {
            CommandBuilder::new("powershell.exe")
        }
    };
    #[cfg(not(target_os = "windows"))]
    let mut cmd = match shell.as_str() {
        "bash" => CommandBuilder::new("/bin/bash"),
        _ => CommandBuilder::new("/bin/sh"),
    };

    cmd.env("TERM", "xterm-256color");

    let mut child = match pty.slave.spawn_command(cmd) {
        Ok(child) => child,
        Err(e) => {
            tracing::error!("启动终端进程失败: {}", e);
            return;
        }
    };
    tracing::info!("终端进程已启动, pid={:?}", child.process_id());

    let reader = pty.master.try_clone_reader().unwrap();
    let writer = pty.master.take_writer().unwrap();

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // PTY reader → WebSocket
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
    tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut reader = reader;
        let mut buf = vec![0u8; 4096];
        let mut first_read = true;
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let mut data = &buf[..n];
                    // 过滤 ConPTY 初始清屏序列，保留 banner 内容
                    if first_read {
                        first_read = false;
                        if let Some(pos) = find_after_clear_screen(data) {
                            data = &data[pos..];
                            if data.is_empty() {
                                continue;
                            }
                        }
                    }
                    if tx.blocking_send(data.to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            if ws_sender.send(Message::Binary(data)).await.is_err() {
                break;
            }
        }
    });

    // WebSocket → PTY writer
    let writer = tokio::sync::Mutex::new(writer);
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if text.starts_with('\x01') {
                        if let Some(json_str) = text.get(1..) {
                            if let Ok(size) = serde_json::from_str::<serde_json::Value>(json_str)
                            {
                                let new_cols = size["cols"].as_u64().unwrap_or(80) as u16;
                                let new_rows = size["rows"].as_u64().unwrap_or(24) as u16;
                                let _ = pty.master.resize(PtySize {
                                    rows: new_rows,
                                    cols: new_cols,
                                    pixel_width: 0,
                                    pixel_height: 0,
                                });
                            }
                        }
                    } else {
                        let mut writer = writer.lock().await;
                        use std::io::Write;
                        let _ = writer.write_all(text.as_bytes());
                        let _ = writer.flush();
                    }
                }
                Message::Binary(data) => {
                    let mut writer = writer.lock().await;
                    use std::io::Write;
                    let _ = writer.write_all(&data);
                    let _ = writer.flush();
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    let _ = child.kill();
    let _ = child.wait();
}
