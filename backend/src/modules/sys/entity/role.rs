use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 角色实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub remark: Option<String>,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}