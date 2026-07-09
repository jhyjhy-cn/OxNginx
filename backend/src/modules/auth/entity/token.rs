use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Token 实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Token {
    pub id: i64,
    pub token: String,
    pub user_id: i64,
    pub username: String,
    pub expires_at: NaiveDateTime,
    pub created_at: Option<NaiveDateTime>,
}