use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::time::Duration;
use tokio::time::Instant;

use crate::modules::websocket::protocol::cmd as Cmd;
use crate::AppState;

/// terminal 通道：独立 ?type=terminal 连接，PTY 双向桥
/// ponytail: 输入改用 cmd 数字协议（cmd:3=terminal.in, cmd:4=resize）；输出仍走 binary 帧
pub async fn serve(socket: WebSocket, _state: AppState, cols: u16, rows: u16, shell: Option<String>) {
    let cols = cols.max(20);
    let rows = rows.max(5);
    let shell = shell.unwrap_or_else(|| "powershell".to_string());
    tracing::info!("[ws:terminal] cols={} rows={} shell={}", cols, rows, shell);

    let pty_system = NativePtySystem::default();
    let pty = match pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(p) => p,
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
        _ => CommandBuilder::new("powershell.exe"),
    };
    #[cfg(not(target_os = "windows"))]
    let mut cmd = match shell.as_str() {
        "bash" => CommandBuilder::new("/bin/bash"),
        _ => CommandBuilder::new("/bin/sh"),
    };
    cmd.env("TERM", "xterm-256color");

    let mut child = match pty.slave.spawn_command(cmd) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("启动终端进程失败: {}", e);
            return;
        }
    };

    // ponytail: pty 已 Ok 验证，try_clone_reader/take_writer 不应再失败
    let reader = pty.master.try_clone_reader().expect("pty reader 不可克隆");
    let writer = pty.master.take_writer().expect("pty writer 不可获取");
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // PTY reader → WS binary（沿用现状，xterm 原生支持）
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
    tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut reader = reader;
        let mut buf = vec![0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.blocking_send(buf[..n].to_vec()).is_err() { break; }
                }
                Err(_) => break,
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            if ws_sender.send(Message::Binary(data.into())).await.is_err() { break; }
        }
    });

    // WS → PTY writer
    let writer = tokio::sync::Mutex::new(writer);
    let mut last_recv = Instant::now();
    let max_idle = Duration::from_secs(600);

    loop {
        tokio::select! {
            biased;
            maybe_msg = ws_receiver.next() => {
                match maybe_msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                        last_recv = Instant::now();
                    }
                    Some(Ok(Message::Text(text))) => {
                        last_recv = Instant::now();
                        // ponytail: 协议层用 cmd 数字；兼容老前端发的 \x01{json} resize 帧
                        if text.starts_with('\x01') {
                            if let Some(json_str) = text.get(1..) {
                                if let Ok(size) = serde_json::from_str::<serde_json::Value>(json_str) {
                                    let nc = size["cols"].as_u64().unwrap_or(80) as u16;
                                    let nr = size["rows"].as_u64().unwrap_or(24) as u16;
                                    let _ = pty.master.resize(PtySize {
                                        rows: nr, cols: nc, pixel_width: 0, pixel_height: 0,
                                    });
                                }
                            }
                            continue;
                        }
                        // 尝试按 cmd 字符串协议解析
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                            match v.get("cmd").and_then(|x| x.as_str()) {
                                Some(s) if s == Cmd::TERMINAL_IN => {
                                    if let Some(data) = v.get("data").and_then(|x| x.as_str()) {
                                        let mut w = writer.lock().await;
                                        use std::io::Write;
                                        let _ = w.write_all(data.as_bytes());
                                        let _ = w.flush();
                                    }
                                }
                                Some(s) if s == Cmd::TERMINAL_RESIZE => {
                                    let nc = v.get("cols").and_then(|x| x.as_u64()).unwrap_or(80) as u16;
                                    let nr = v.get("rows").and_then(|x| x.as_u64()).unwrap_or(24) as u16;
                                    let _ = pty.master.resize(PtySize {
                                        rows: nr, cols: nc, pixel_width: 0, pixel_height: 0,
                                    });
                                }
                                _ => {
                                    // 兜底：原样写入 PTY
                                    let mut w = writer.lock().await;
                                    use std::io::Write;
                                    let _ = w.write_all(text.as_bytes());
                                    let _ = w.flush();
                                }
                            }
                        } else {
                            // 非 JSON 文本：原样写入
                            let mut w = writer.lock().await;
                            use std::io::Write;
                            let _ = w.write_all(text.as_bytes());
                            let _ = w.flush();
                        }
                    }
                    Some(Ok(Message::Binary(data))) => {
                        last_recv = Instant::now();
                        let mut w = writer.lock().await;
                        use std::io::Write;
                        let _ = w.write_all(&data);
                        let _ = w.flush();
                    }
                    Some(Err(_)) => break,
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                if last_recv.elapsed() > max_idle { break; }
            }
        }
    }

    let _ = send_task.await;
    let _ = child.kill();
    let _ = child.wait();
}