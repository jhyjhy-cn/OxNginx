use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 访问控制规则实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccessRule {
    pub id: i64,
    pub site_id: Option<i64>,
    pub rule_type: String,
    pub value: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
}