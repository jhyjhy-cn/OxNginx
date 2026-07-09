use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 配置模板实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub config: String,
    pub variables: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}