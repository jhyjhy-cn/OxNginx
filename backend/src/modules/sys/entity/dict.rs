use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 字典
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dict {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// 字典项
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DictItem {
    pub id: i64,
    pub dict_id: i64,
    pub label: String,
    pub value: String,
    pub sort: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}