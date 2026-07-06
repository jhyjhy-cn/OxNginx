use std::process::Command;
use tokio::process::Command as TokioCommand;

/// Windows: 隐藏窗口标志
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 创建同步命令（隐藏窗口）
pub fn silent_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    set_no_window(&mut cmd);
    cmd
}

/// 创建异步命令（隐藏窗口）
pub fn silent_tokio_command(program: &str) -> TokioCommand {
    let mut cmd = TokioCommand::new(program);
    set_no_window_async(&mut cmd);
    cmd
}

/// 设置同步命令隐藏窗口
#[cfg(target_os = "windows")]
fn set_no_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn set_no_window(_cmd: &mut Command) {}

/// 设置异步命令隐藏窗口
#[cfg(target_os = "windows")]
#[allow(unused_imports)]
fn set_no_window_async(cmd: &mut TokioCommand) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn set_no_window_async(_cmd: &mut TokioCommand) {}

