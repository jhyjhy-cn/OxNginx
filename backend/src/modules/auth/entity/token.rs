use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::{naive_datetime, option_naive_datetime};

/// Token 实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Token {
    pub id: i64,
    pub token: String,
    pub user_id: i64,
    pub username: String,
    pub ip: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub user_agent: Option<String>,
    #[serde(with = "naive_datetime")]
    pub expires_at: NaiveDateTime,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
}
