use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 用户实体
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub nickname: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub remark: Option<String>,
    pub dept_id: Option<i64>,
    pub post_id: Option<i64>,
    pub disabled: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}