use serde::{Deserialize, Serialize};

/// 字符串 cmd 标识：可读性好，调试方便
#[allow(dead_code)]
pub mod cmd {
    // 客户端 → 服务端
    pub const PING: &str = "ping";
    pub const SUBSCRIBE: &str = "subscribe";
    pub const UNSUBSCRIBE: &str = "unsubscribe";
    pub const TERMINAL_IN: &str = "terminal.in";
    pub const TERMINAL_RESIZE: &str = "terminal.resize";
    /// ponytail: terminal 启动握手帧（替代 ?type=terminal 路径里的 query 参数）
    pub const TERMINAL_INIT: &str = "terminal.init";

    // 服务端 → 客户端
    pub const PONG: &str = "pong";
    pub const DASHBOARD: &str = "dashboard";
    pub const EVENT: &str = "event";
}

/// 客户端订阅的通道
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Dashboard,
    Events,
}

impl Channel {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "dashboard" => Some(Self::Dashboard),
            "events" => Some(Self::Events),
            _ => None,
        }
    }
}

/// 客户端入站帧：{cmd, ...payload}
/// ponytail: serde tag 按 cmd 字符串字面量分发到具体变体
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "cmd")]
pub enum ClientFrame {
    #[serde(rename = "ping")]
    Ping {},
    #[serde(rename = "subscribe")]
    Subscribe { channels: Vec<String> },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { channels: Vec<String> },
    #[serde(rename = "terminal.in")]
    TerminalIn { data: String },
    #[serde(rename = "terminal.resize")]
    TerminalResize { cols: u16, rows: u16 },
    #[serde(rename = "terminal.init")]
    TerminalInit {
        cols: Option<u16>,
        rows: Option<u16>,
        shell: Option<String>,
    },
}

/// 客户端入站帧 cmd 字段取值（用于日志）
impl ClientFrame {
    #[allow(dead_code)]
    pub fn cmd(&self) -> &str {
        match self {
            Self::Ping {} => "ping",
            Self::Subscribe { .. } => "subscribe",
            Self::Unsubscribe { .. } => "unsubscribe",
            Self::TerminalIn { .. } => "terminal.in",
            Self::TerminalResize { .. } => "terminal.resize",
            Self::TerminalInit { .. } => "terminal.init",
        }
    }
}

/// 业务事件 payload
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerEvent {
    /// 强退：踢掉指定 token
    Kick { token_id: i64, #[serde(default)] reason: i32 },
    Notice { message: String },
}

/// 服务端出站帧（dashboard + events）
#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "cmd", content = "payload")]
pub enum ServerFrame<'a> {
    #[serde(rename = "dashboard")]
    Dashboard(&'a serde_json::Value),
    #[serde(rename = "event")]
    Event(&'a ServerEvent),
    #[serde(rename = "pong")]
    Pong,
}