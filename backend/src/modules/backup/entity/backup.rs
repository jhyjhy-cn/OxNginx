use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 备份实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Backup {
    pub id: i64,
    pub site_id: Option<i64>,
    pub version: i32,
    pub config: String,
    pub created_at: Option<NaiveDateTime>,
}