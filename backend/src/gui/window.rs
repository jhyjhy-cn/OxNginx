use std::sync::{Arc, Mutex, OnceLock, mpsc};
use eframe::egui;
use crate::config::AppConfig;

/// 加载窗口图标数据
fn load_window_icon() -> Arc<egui::IconData> {
    // 开发环境：直接用相对路径
    // 打包环境：从exe目录加载
    let mut ico_paths = vec![
        std::path::PathBuf::from("assets/icons/logo_128x128.ico"),
    ];

    // 如果相对路径找不到，尝试从exe目录加载
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(dir) = exe_dir {
        ico_paths.push(dir.join("assets/icons/logo_128x128.ico"));
    }

    for path in &ico_paths {
        match std::fs::read(path) {
            Ok(bytes) => {
                match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let (width, height) = rgba.dimensions();
                        let icon = egui::IconData {
                            rgba: rgba.into_raw(),
                            width,
                            height,
                        };
                        tracing::info!("窗口图标加载成功: {}", path.display());
                        return Arc::new(icon);
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    }

    tracing::warn!("使用默认窗口图标");
    Arc::new(egui::IconData {
        rgba: vec![0, 70, 150, 255],
        width: 1,
        height: 1,
    })
}

/// 全局发送器，用于从托盘线程发送显示命令
pub(crate) static WINDOW_SENDER: OnceLock<mpsc::Sender<WindowCommand>> = OnceLock::new();

/// 窗口命令
pub(crate) enum WindowCommand {
    Show,
}

/// OxNginx 控制面板应用
pub struct OxNginxApp {
    /// 服务是否运行
    service_running: Arc<Mutex<bool>>,
    /// 配置
    config: AppConfig,
    /// 运行时间（秒）
    uptime_seconds: u64,
    /// 内存占用（字节）
    memory_usage: u64,
    /// 最后更新时间
    last_update: std::time::Instant,
    /// 命令接收器
    receiver: mpsc::Receiver<WindowCommand>,
}

impl OxNginxApp {
    pub fn new(config: AppConfig, service_running: Arc<Mutex<bool>>, receiver: mpsc::Receiver<WindowCommand>) -> Self {
        Self {
            service_running,
            config,
            uptime_seconds: 0,
            memory_usage: 0,
            last_update: std::time::Instant::now(),
            receiver,
        }
    }

    /// 格式化运行时间
    fn format_uptime(&self) -> String {
        let hours = self.uptime_seconds / 3600;
        let minutes = (self.uptime_seconds % 3600) / 60;
        let seconds = self.uptime_seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// 格式化内存占用
    fn format_memory(&self) -> String {
        let mb = self.memory_usage as f64 / 1024.0 / 1024.0;
        format!("{:.1} MB", mb)
    }

    /// 获取面板地址
    fn get_panel_url(&self) -> String {
        let host = if self.config.server.host == "0.0.0.0" {
            "localhost"
        } else {
            &self.config.server.host
        };
        format!("http://{}:{}", host, self.config.server.port)
    }
}

impl eframe::App for OxNginxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查是否有来自托盘的命令
        match self.receiver.try_recv() {
            Ok(WindowCommand::Show) => {
                tracing::info!("收到显示窗口命令");
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            Err(_) => {}
        }

        // 每秒更新一次数据
        let now = std::time::Instant::now();
        if now.duration_since(self.last_update).as_secs() >= 1 {
            self.uptime_seconds += 1;
            self.last_update = now;

            // TODO: 从服务获取实际内存占用
            self.memory_usage = 15 * 1024 * 1024; // 15MB

            // 请求重绘
            ctx.request_repaint();
        }

        // 绘制主面板
        egui::CentralPanel::default().show(ctx, |ui| {
            // 标题
            ui.horizontal(|ui| {
                ui.heading("OxNginx Manager");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                });
            });

            ui.separator();

            // 服务状态卡片
            let running = *self.service_running.lock().unwrap();
            let (status_text, status_color) = if running {
                ("● 服务运行中", egui::Color32::from_rgb(0, 180, 0))
            } else {
                ("● 服务已停止", egui::Color32::from_rgb(200, 0, 0))
            };

            egui::Frame::none()
                .fill(if running {
                    egui::Color32::from_rgb(232, 245, 233)
                } else {
                    egui::Color32::from_rgb(255, 235, 238)
                })
                .rounding(8.0)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.colored_label(status_color, egui::RichText::new(status_text).size(18.0));
                });

            ui.add_space(16.0);

            // 面板地址
            ui.label("面板地址:");
            ui.horizontal(|ui| {
                let url = self.get_panel_url();
                let url_label = ui.label(
                    egui::RichText::new(&url)
                        .color(egui::Color32::from_rgb(33, 150, 243))
                        .strong()
                );

                if url_label.clicked() {
                    let _ = open::that(&url);
                }

                if ui.button("复制").clicked() {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = clipboard_win::set_clipboard_string(&url);
                    }
                }

                if ui.button("打开").clicked() {
                    let _ = open::that(&url);
                }
            });

            ui.add_space(16.0);

            // 控制按钮
            ui.horizontal(|ui| {
                if running {
                    if ui.button("停止服务").clicked() {
                        tracing::info!("用户点击: 停止服务");
                    }
                } else {
                    if ui.button("启动服务").clicked() {
                        tracing::info!("用户点击: 启动服务");
                    }
                }

                if ui.button("重启服务").clicked() {
                    tracing::info!("用户点击: 重启服务");
                }
            });

            ui.add_space(16.0);

            // 统计信息
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("运行时间:");
                ui.strong(self.format_uptime());

                ui.add_space(32.0);

                ui.label("内存占用:");
                ui.strong(self.format_memory());
            });

            // 底部提示
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("关闭窗口将最小化到系统托盘")
                        .color(egui::Color32::GRAY)
                        .small()
                );
            });
        });

        // 拦截关闭请求 -最小化到托盘而不是退出
        if ctx.input(|i| i.viewport().close_requested()) {
            // 隐藏窗口而不是关闭
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            tracing::info!("窗口最小化到托盘");
        }
    }
}

/// 运行窗口（阻塞当前线程，使用外部传入的receiver）
pub fn run_window_with_receiver(config: AppConfig, service_running: Arc<Mutex<bool>>, receiver: mpsc::Receiver<WindowCommand>) {
    tracing::info!("run_window_with_receiver被调用，准备创建窗口");

    // 加载图标用于窗口
    let icon_data = load_window_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 320.0])
            .with_min_inner_size([350.0, 280.0])
            .with_resizable(true)
            .with_icon(icon_data),
        ..Default::default()
    };

    tracing::info!("创建OxNginxApp");
    let app = OxNginxApp::new(config, service_running, receiver);

    tracing::info!("调用eframe::run_native");
    match eframe::run_native(
        "OxNginx Manager",
        options,
        Box::new(|cc| {
            tracing::info!("eframe初始化回调 - 加载中文字体");
            // 加载中文字体
            load_chinese_font(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    ) {
        Ok(_) => tracing::info!("eframe正常退出"),
        Err(e) => tracing::error!("eframe错误: {}", e),
    }
    tracing::info!("run_window结束");
}

/// 显示窗口（从托盘线程调用）-使用 Windows API
pub fn show_window() {
    if let Some(sender) = WINDOW_SENDER.get() {
        let _ = sender.send(WindowCommand::Show);
        tracing::info!("已发送显示窗口命令");
    } else {
        tracing::warn!("WINDOW_SENDER未初始化");
    }

    // 使用 Windows API 强制显示窗口
    #[cfg(target_os = "windows")]
    {
        use winapi::um::winuser::{FindWindowW, ShowWindow, SetForegroundWindow, SW_SHOW};
        use std::ptr::null_mut;

        unsafe {
            // 查找窗口（通过标题）
            let title: Vec<u16> = "OxNginx Manager\0".encode_utf16().collect();
            let hwnd = FindWindowW(null_mut(), title.as_ptr());
            if !hwnd.is_null() {
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
                tracing::info!("通过Windows API显示窗口");
            } else {
                tracing::warn!("未找到窗口");
            }
        }
    }
}

/// 加载中文字体
fn load_chinese_font(ctx: &egui::Context) {
    // 使用egui内置的中文字体支持
    let mut fonts = egui::FontDefinitions::default();

    // 尝试加载系统中文字体（Windows）
    #[cfg(target_os = "windows")]
    {
        // 微软雅黑字体路径
        let font_paths = [
            "C:/Windows/Fonts/msyh.ttc",      // 微软雅黑
            "C:/Windows/Fonts/simhei.ttf",     // 黑体
            "C:/Windows/Fonts/simsun.ttc",     // 宋体
        ];

        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                tracing::info!("加载字体: {}", font_path);
                fonts.font_data.insert(
                    "chinese".to_owned(),
                    egui::FontData::from_owned(font_data),
                );

                // 将中文字体添加到所有字体族的最后（作为fallback）
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                    fonts.families
                        .entry(family)
                        .or_default()
                        .push("chinese".to_owned());
                }
                break;
            }
        }
    }

    ctx.set_fonts(fonts);
}
