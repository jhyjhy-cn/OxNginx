use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem, MenuEvent, PredefinedMenuItem},
    Icon,
};

/// 加载图标 - 使用logo.ico文件
fn load_icon() -> Icon {
    // 开发环境：直接用相对路径（从backend目录运行）
    // 打包环境：从exe目录加载
    let mut ico_paths = vec![
        std::path::PathBuf::from("assets/icons/logo_32x32.ico"),
        std::path::PathBuf::from("assets/icons/logo_48x48.ico"),
    ];

    // 如果相对路径找不到，尝试从exe目录加载
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(dir) = exe_dir {
        ico_paths.extend([
            dir.join("assets/icons/logo_32x32.ico"),
            dir.join("assets/icons/logo_48x48.ico"),
        ]);
    }

    for path in &ico_paths {
        match load_ico_icon(path) {
            Ok(icon) => {
                tracing::info!("成功加载图标: {}", path.display());
                return icon;
            }
            Err(_) => {}
        }
    }

    tracing::warn!("无法加载自定义图标，使用默认图标");
    create_default_icon()
}

/// 从 ICO文件加载图标
fn load_ico_icon(path: &std::path::Path) -> Result<Icon, Box<dyn std::error::Error>> {
    let ico_bytes = std::fs::read(path)?;
    let img = image::load_from_memory(&ico_bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(Icon::from_rgba(rgba.into_raw(), width, height)?)
}

/// 创建默认图标（OxNginx蓝色）
fn create_default_icon() -> Icon {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    // 绘制一个简单的"N"字形图标
    for y in 0..height {
        for x in 0..width {
            let offset = ((y * width + x) * 4) as usize;

            // 背景色 - 深蓝
            let mut r = 0;
            let mut g = 70;
            let mut b = 150;

            // 绘制字母"N"
            let in_left = x >= 8 && x <= 12;
            let in_right = x >= 20 && x <= 24;
            let in_top = y >= 4 && y <= 8;
            let in_bottom = y >= 24 && y <= 28;
            let in_diag = (x as i32 - y as i32 + 4).abs() <= 3 && x >= 8 && x <= 24;

            if (in_left || in_right) || (in_top || in_bottom) || in_diag {
                r = 255;
                g = 255;
                b = 255;
            }

            rgba[offset] = r;
            rgba[offset + 1] = g;
            rgba[offset + 2] = b;
            rgba[offset + 3] = 255;
        }
    }

    Icon::from_rgba(rgba, width, height).expect("Failed to create default icon")
}

/// 创建托盘图标（必须在主线程调用，返回图标保持存活）
pub fn create_tray_icon(config: &crate::config::AppConfig) -> anyhow::Result<tray_icon::TrayIcon> {
    // 创建托盘菜单
    let tray_menu = Menu::new();

    // 面板操作
    let show_item = MenuItem::new("显示窗口", true, None);
    let open_item = MenuItem::new("打开面板", true, None);
    let copy_item = MenuItem::new("复制地址", true, None);
    let quit_item = MenuItem::new("退出", true, None);

    tray_menu.append(&show_item)?;
    tray_menu.append(&PredefinedMenuItem::separator())?;
    tray_menu.append(&open_item)?;
    tray_menu.append(&copy_item)?;
    tray_menu.append(&PredefinedMenuItem::separator())?;
    tray_menu.append(&quit_item)?;

    // 保存菜单项ID用于事件匹配
    let show_id = show_item.id().0.clone();
    let open_id = open_item.id().0.clone();
    let copy_id = copy_item.id().0.clone();
    let quit_id = quit_item.id().0.clone();

    // 创建托盘图标
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("OxNginx Manager - 右键打开菜单")
        .with_icon(load_icon())
        .build()?;

    // 启动事件处理线程
    let menu_channel = MenuEvent::receiver();
    let config_clone = config.clone();

    std::thread::spawn(move || {
        tracing::info!("托盘事件处理线程启动");
        loop {
            if let Ok(event) = menu_channel.recv_timeout(std::time::Duration::from_millis(100)) {
                let id = event.id.0.as_str();
                tracing::info!("托盘菜单事件: {}", id);

                if id == show_id.as_str() {
                    tracing::info!("菜单: 显示窗口");
                    // 调用window模块的显示函数
                    super::window::show_window();
                } else if id == open_id.as_str() {
                    let url = format!("http://localhost:{}", config_clone.server.port);
                    let _ = open::that(&url);
                } else if id == copy_id.as_str() {
                    let addr = format!("http://localhost:{}", config_clone.server.port);
                    #[cfg(target_os = "windows")]
                    let _ = clipboard_win::set_clipboard_string(&addr);
                } else if id == quit_id.as_str() {
                    std::process::exit(0);
                }
            }
        }
    });

    tracing::info!("系统托盘创建成功");
    Ok(tray_icon)
}
