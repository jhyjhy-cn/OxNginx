pub mod dashboard_push;
pub mod hub;
pub mod protocol;
pub mod terminal;

use axum::{
    extract::{ws::WebSocketUpgrade, Extension, Query, State},
    response::Response,
};
use serde::Deserialize;

use crate::modules::common::middleware::TokenInfo;
use crate::AppState;

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct WsQuery {
    pub token: String,
    /// ponytail: ?type=terminal 时走 PTY；缺省走 dashboard+events 通用 hub
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub cols: Option<u16>,
    #[serde(default)]
    pub rows: Option<u16>,
    #[serde(default)]
    pub shell: Option<String>,
}

/// 通用 ws（dashboard + events 共用）—— auth_middleware 已鉴权
pub async fn ws_entry(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
    Query(q): Query<WsQuery>,
) -> Response {
    if q.r#type.as_deref() == Some("terminal") {
        let cols = q.cols.unwrap_or(80);
        let rows = q.rows.unwrap_or(24);
        let shell = q.shell.clone();
        return ws.on_upgrade(move |socket| {
            terminal::serve(socket, state, cols, rows, shell)
        });
    }
    let token_id = info.token_id;
    ws.on_upgrade(move |socket| hub::serve(socket, state, token_id))
}