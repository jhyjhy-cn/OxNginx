use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 国际化翻译条目
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct I18nEntry {
    pub id: i64,
    pub locale: String,
    pub key: String,
    pub value: String,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}