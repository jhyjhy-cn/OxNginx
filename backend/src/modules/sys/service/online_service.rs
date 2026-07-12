use anyhow::Result;
use sqlx::SqlitePool;

use crate::modules::auth::service::token_service;
use crate::modules::sys::dao::online_dao::{self, OnlineItem};
use crate::modules::websocket::protocol::ServerEvent;
use crate::AppState;

/// 分页查询在线会话
pub async fn list_online(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> Result<(Vec<OnlineItem>, i64)> {
    Ok(online_dao::list_online_paged(pool, page, page_size, keyword).await?)
}

/// 强退指定会话；删完 token 后广播 Kick，前端 ws 收到即跳转登录
pub async fn kick_online(state: &AppState, id: i64) -> Result<bool> {
    let ok = token_service::delete_token_by_id(state.db.pool(), id).await?;
    if ok {
        let _ = state.event_tx.send(ServerEvent::Kick {
            token_id: id,
            reason: 0,
        });
    }
    Ok(ok)
}