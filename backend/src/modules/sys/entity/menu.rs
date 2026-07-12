use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::modules::common::util::datetime::option_naive_datetime;

/// 菜单/按钮（type: 1=目录 2=菜单 3=按钮）
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Menu {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub title: String,
    pub icon: Option<String>,
    pub path: Option<String>,
    pub component: Option<String>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub menu_type: i32,
    pub permission: Option<String>,
    pub sort: i32,
    pub status: i32, // 1=启用 0=禁用
    pub version: i32,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub remark: Option<String>,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}