use std::sync::OnceLock;

use tokio::sync::mpsc;

use crate::audit::event::AuditEvent;

static TX: OnceLock<mpsc::Sender<AuditEvent>> = OnceLock::new();

/// 初始化 mpsc sender，返回 receiver 供 worker 持有。
/// 只在 main 启动时调用一次。
pub fn init() -> mpsc::Receiver<AuditEvent> {
    let (tx, rx) = mpsc::channel(4096);
    TX.set(tx).expect("audit sender inited");
    rx
}

/// 提交一条审计事件。
/// - module 和 action 都为空时跳过（未标 `#[audit_log]` 的接口）
/// - channel 满时 `send().await` 自然背压，不丢日志
pub async fn submit(ev: AuditEvent) {
    if ev.module.is_empty() && ev.action.is_empty() {
        return;
    }
    if let Some(tx) = TX.get() {
        if let Err(e) = tx.send(ev).await {
            tracing::error!("audit submit fail (channel closed): {}", e);
        }
    }
}
