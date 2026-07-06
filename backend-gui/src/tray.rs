use tauri::{
    AppHandle, Manager,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem},
};

/// 创建系统托盘
pub fn create_tray(app: &AppHandle) -> anyhow::Result<()> {
    // 创建菜单
    let show_item = MenuItem::with_id(app, "show", "显示状态", true, None::<&str>)?;
    let open_item = MenuItem::with_id(app, "open", "打开面板", true, None::<&str>)?;
    let copy_item = MenuItem::with_id(app, "copy", "复制地址", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &open_item, &copy_item, &quit_item])?;

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("OxNginx Manager")
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "show" => {
                    // 显示状态窗口
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "open" => {
                    // 打开浏览器访问面板
                    let _ = open::that("http://localhost:9000");
                }
                "copy" => {
                    // 复制地址到剪贴板
                    #[cfg(target_os = "windows")]
                    {
                        let _ = clipboard_win::set_clipboard_string("http://localhost:9000");
                    }
                }
                "quit" => {
                    // 停止 backend 进程
                    super::process::stop_backend();
                    // 退出应用
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // 左键点击显示状态窗口
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
