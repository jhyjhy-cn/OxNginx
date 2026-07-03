use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ========== 自定义日志格式：时间 级别 文件:行号 消息 ==========

struct LocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();
        let formatted = format!("{}", now.format("%Y-%m-%d %H:%M:%S%.3f%:z"));
        write!(w, "{}", formatted)
    }
}

/// 辅助：将 tracing Event 的字段写入 Writer（key1=value1 key2=value2 ...）
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

/// 完全自定义的事件格式：时间 级别 文件:行号 消息
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
        write!(writer, " {} {}:{} : ", level_str, file, line)?;

        let mut field_writer = EventFieldWriter {
            writer: &mut writer,
            is_first: true,
        };
        event.record(&mut field_writer);
        writeln!(writer)
    }
}

/// 初始化 tracing 日志系统
pub fn init() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ox_nginx=debug,tower_http=debug".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(FullFormat {
                    timer: LocalTimer,
                    ansi: true,
                })
                .with_ansi(true),
        )
        .init();
}
