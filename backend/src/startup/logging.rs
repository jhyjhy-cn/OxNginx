use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
            .file_stem().and_then(|s| s.to_str()).unwrap_or("panel");
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
/// ponytail: parking_lot::Mutex 无 poison；append 失败退化为 NUL 文件而不是 panic
struct SizeBasedAppender {
    inner: parking_lot::Mutex<SizeBasedAppenderInner>,
    written: Arc<parking_lot::Mutex<u64>>,
}

impl SizeBasedAppender {
    fn new(dir: PathBuf, base_name: &str, max_mb: u64) -> Self {
        let current_path = dir.join(base_name);
        let current_bytes = std::fs::metadata(&current_path).map(|m| m.len()).unwrap_or(0);
        SizeBasedAppender {
            inner: parking_lot::Mutex::new(SizeBasedAppenderInner {
                dir,
                base_name: base_name.to_string(),
                max_bytes: max_mb * 1024 * 1024,
            }),
            written: Arc::new(parking_lot::Mutex::new(current_bytes)),
        }
    }
}

struct CountingFile {
    inner: std::fs::File,
    written: Arc<parking_lot::Mutex<u64>>,
}

impl std::io::Write for CountingFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        *self.written.lock() += n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> { self.inner.flush() }
}

impl Drop for CountingFile {
    fn drop(&mut self) { let _ = self.inner.flush(); }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SizeBasedAppender {
    type Writer = CountingFile;
    fn make_writer(&'a self) -> Self::Writer {
        let mut inner = self.inner.lock();
        if *self.written.lock() >= inner.max_bytes {
            if let Err(e) = inner.rotate() { eprintln!("日志轮转失败: {}", e); }
            *self.written.lock() = 0;
        }
        let file = match std::fs::OpenOptions::new().create(true).append(true)
            .open(inner.current_path())
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("无法打开日志文件: {}", e);
                // ponytail: 退到 NUL 静默丢弃，避免 panic 拖死进程
                std::fs::File::create("NUL").unwrap_or_else(|_| std::fs::File::create("/dev/null").expect("NUL/dev/null 都开不了"))
            }
        };
        CountingFile { inner: file, written: Arc::clone(&self.written) }
    }
}

// ========== 自定义日志格式 ==========

struct LocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f%:z"))
    }
}

struct EventFieldWriter<'a, 'b> {
    writer: &'a mut tracing_subscriber::fmt::format::Writer<'b>,
    is_first: bool,
}

impl tracing::field::Visit for EventFieldWriter<'_, '_> {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if self.is_first {
            let _ = write!(self.writer, "{:?}", value);
            self.is_first = false;
        } else {
            let _ = write!(self.writer, " {:?}", value);
        }
    }
}

// 线程局部：第一个出现的线程 ID 标记为 "main"
thread_local!(static MAIN_TID: std::cell::RefCell<Option<std::thread::ThreadId>> = const { std::cell::RefCell::new(None) });

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
        let loc = if meta.target().starts_with("sqlx") {
            format!("sqlx:{}", line)
        } else {
            format!("{}:{}", file, line)
        };

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

        let tid = std::thread::current().id();
        let tid_str = MAIN_TID.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_none() { *slot = Some(tid); }
            if slot.as_ref() == Some(&tid) { "main".to_string() } else { format!("{:?}", tid) }
        });
        write!(writer, " {} [{}] {}: ", level_str, tid_str, loc)?;

        let mut field_writer = EventFieldWriter { writer: &mut writer, is_first: true };
        event.record(&mut field_writer);
        writeln!(writer)
    }
}

// ========== 初始化 ==========

pub fn init(log_dir: &Path, log_level: &str, max_size_mb: u64, log_sql: bool) {
    let _ = std::fs::create_dir_all(log_dir);
    cleanup_old_logs(log_dir, 30);

    let appender = SizeBasedAppender::new(log_dir.to_path_buf(), "panel.log", max_size_mb);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .event_format(FullFormat { timer: LocalTimer, ansi: true })
        .with_ansi(true);

    let file_layer = tracing_subscriber::fmt::layer()
        .event_format(FullFormat { timer: LocalTimer, ansi: false })
        .with_ansi(false)
        .with_writer(appender);

    let sqlx_level = if log_sql { "debug" } else { "off" };
    let filter = format!("ox_nginx={},tower_http=warn,sqlx={}", log_level, sqlx_level);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(std::env::var("RUST_LOG").unwrap_or(filter)))
        .with(stdout_layer)
        .with(file_layer)
        .try_init().ok(); // 幂等初始化，第二次调用会忽略
}

fn cleanup_old_logs(log_dir: &Path, retention_days: i64) {
    let cutoff = chrono::Local::now() - chrono::Duration::days(retention_days);
    let cutoff_str = cutoff.format("%Y%m%d").to_string();
    let entries = match std::fs::read_dir(log_dir) { Ok(e) => e, Err(_) => return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if let Some(rest) = name_str.strip_prefix("panel-") {
            if let Some(date_part) = rest.get(..8) {
                if date_part < cutoff_str.as_str() { let _ = std::fs::remove_file(entry.path()); }
            }
        }
    }
}
