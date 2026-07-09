use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 国际化翻译条目
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct I18nEntry {
    pub id: i64,
    pub locale: String,
    pub key: String,
    pub value: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}