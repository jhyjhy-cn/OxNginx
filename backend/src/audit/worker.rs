use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::mpsc;
use tokio::time::{interval, MissedTickBehavior};

use crate::audit::event::AuditEvent;
use crate::service::log_service;

/// 启动后台 batch flush worker。
/// 攒够 50 条或 500ms tick 触发一次 flush。
/// flush 失败时降级逐条插入。
pub fn spawn(rx: mpsc::Receiver<AuditEvent>, pool: SqlitePool) {
    tokio::spawn(async move {
        let mut rx = rx;
        let mut buf: Vec<AuditEvent> = Vec::with_capacity(64);
        let mut tick = interval(Duration::from_millis(500));
        tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                Some(ev) = rx.recv() => {
                    buf.push(ev);
                    if buf.len() >= 50 {
                        flush(&pool, &mut buf).await;
                    }
                }
                _ = tick.tick() => {
                    if !buf.is_empty() {
                        flush(&pool, &mut buf).await;
                    }
                }
                else => break,
            }
        }
        if !buf.is_empty() {
            flush(&pool, &mut buf).await;
        }
        tracing::info!("audit worker exited");
    });
}

async fn flush(pool: &SqlitePool, buf: &mut Vec<AuditEvent>) {
    if buf.is_empty() {
        return;
    }
    match log_service::log_operations_batch_multirow(pool, buf).await {
        Ok(_) => buf.clear(),
        Err(e) => {
            tracing::error!(
                "audit batch insert fail ({} events): {} — fallback to single",
                buf.len(),
                e
            );
            let drained: Vec<AuditEvent> = buf.drain(..).collect();
            for ev in drained {
                if let Err(e2) = log_service::log_operation_single(pool, &ev).await {
                    tracing::error!("audit single insert fail: {} — drop event", e2);
                }
            }
        }
    }
}
