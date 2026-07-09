use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 岗位实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub sort: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}