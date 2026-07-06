use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::io::Write;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ========== 按大小轮转的日志文件 ==========

struct SizeBasedAppenderInner {
    dir: PathBuf,
    base_name: String,
    max_bytes: u64,
}

impl SizeBasedAppenderInner {
    fn current_path(&self) -> PathBuf {
        self.dir.join(&self.base_name)
    }

    fn rotate(&mut self) -> std::io::Result<()> {
        let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let stem = Path::new(&self.base_name)
            .file_stem().and_then(|s| s.to_str()).unwrap_or("gui");
        let ext = Path::new(&self.base_name)
            .extension().and_then(|e| e.to_str()).unwrap_or("log");
        let rotated = self.dir.join(format!("{}-{}.{}", stem, ts, ext));

        let current = self.current_path();
        if current.exists() {
            std::fs::rename(&current, &rotated)?;
        }
        Ok(())
    }
}

/// 按文件大小自动轮转（线程安全）
struct SizeBasedAppender {
    inner: Mutex<SizeBasedAppenderInner>,
    written: Arc<Mutex<u64>>,
}

impl SizeBasedAppender {
    fn new(dir: PathBuf, base_name: &str, max_mb: u64) -> Self {
        let current_path = dir.join(base_name);
        let current_bytes = std::fs::metadata(&current_path)
            .map(|m| m.len()).unwrap_or(0);

        SizeBasedAppender {
            inner: Mutex::new(SizeBasedAppenderInner {
                dir,
                base_name: base_name.to_string(),
                max_bytes: max_mb * 1024 * 1024,
            }),
            written: Arc::new(Mutex::new(current_bytes)),
        }
    }
}

struct CountingFile {
    inner: std::fs::File,
    written: Arc<Mutex<u64>>,
}

impl std::io::Write for CountingFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        if let Ok(mut w) = self.written.lock() {
            *w += n as u64;
        }
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl Drop for CountingFile {
    fn drop(&mut self) {
        let _ = self.inner.flush();
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SizeBasedAppender {
    type Writer = CountingFile;

    fn make_writer(&'a self) -> Self::Writer {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());

        if *self.written.lock().unwrap_or_else(|e| e.into_inner()) >= inner.max_bytes {
            if let Err(e) = inner.rotate() {
                eprintln!("日志轮转失败: {}", e);
            }
            *self.written.lock().unwrap_or_else(|e| e.into_inner()) = 0;
        }

        let file = std::fs::OpenOptions::new()
            .create(true).append(true)
            .open(inner.current_path())
            .unwrap_or_else(|e| {
                eprintln!("无法打开日志文件: {}", e);
                std::fs::File::create("NUL").unwrap()
            });

        CountingFile { inner: file, written: Arc::clone(&self.written) }
    }
}

// ========== 自定义日志格式 ==========

struct LocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f%:z"))
    }
}

struct FullFormat {
    timer: LocalTimer,
    ansi: bool,
}

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for FullFormat
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();
        let level = meta.level();
        let file = meta.file().unwrap_or("?");
        let line = meta.line().unwrap_or(0);

        let level_str = match *level {
            tracing::Level::ERROR if self.ansi => "\x1b[31mERROR\x1b[0m",
            tracing::Level::WARN if self.ansi => "\x1b[33mWARN \x1b[0m",
            tracing::Level::INFO if self.ansi => "\x1b[32mINFO \x1b[0m",
            tracing::Level::DEBUG if self.ansi => "\x1b[36mDEBUG\x1b[0m",
            tracing::Level::TRACE if self.ansi => "\x1b[35mTRACE\x1b[0m",
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN => "WARN ",
            tracing::Level::INFO => "INFO ",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "TRACE",
        };

        use tracing_subscriber::fmt::time::FormatTime;
        self.timer.format_time(&mut writer)?;
        write!(writer, " {} {}:{}: ", level_str, file, line)?;

        // 写入消息字段
        let mut visitor = MessageVisitor { writer: &mut writer, is_first: true };
        event.record(&mut visitor);
        writeln!(writer)
    }
}

struct MessageVisitor<'a, 'b> {
    writer: &'a mut tracing_subscriber::fmt::format::Writer<'b>,
    is_first: bool,
}

impl tracing::field::Visit for MessageVisitor<'_, '_> {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if self.is_first {
            let _ = write!(self.writer, "{:?}", value);
            self.is_first = false;
        } else {
            let _ = write!(self.writer, " {:?}", value);
        }
    }
}

// ========== 初始化 ==========

/// 初始化日志系统（控制台 + 文件双输出）
pub fn init() {
    let log_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("wwwlogs").join("gui");

    let _ = std::fs::create_dir_all(&log_dir);

    let appender = SizeBasedAppender::new(log_dir.clone(), "gui.log", 10); // 10MB

    let stdout_layer = tracing_subscriber::fmt::layer()
        .event_format(FullFormat { timer: LocalTimer, ansi: true })
        .with_ansi(true);

    let file_layer = tracing_subscriber::fmt::layer()
        .event_format(FullFormat { timer: LocalTimer, ansi: false })
        .with_ansi(false)
        .with_writer(appender);

    let filter = "ox_nginx_gui=info";

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| filter.to_string()),
        ))
        .with(stdout_layer)
        .with(file_layer)
        .init();

    tracing::info!("日志目录: {:?}", log_dir);
}
