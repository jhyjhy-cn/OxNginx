use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 部门实体
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Dept {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}