use parking_lot::Mutex;
use std::sync::Arc;

/// 单次请求的审计上下文。
/// 字段在 handler 内部由 `#[audit_log]` 宏写入，handler 返回后由中间件读取。
#[derive(Default, Clone, Debug)]
pub struct AuditContext {
    pub module: Option<String>,
    pub action: Option<String>,
    pub params: Option<String>,
    pub error: Option<String>,
}

/// 跨 handler / 中间件共享的审计上下文句柄类型。
/// 中间件把它塞进 `request.extensions_mut()`，handler 用 `Extension<SharedAuditContext>` 拿。
/// ponytail: parking_lot::Mutex 无 poison，无锁毒崩溃
pub type SharedAuditContext = Arc<Mutex<AuditContext>>;