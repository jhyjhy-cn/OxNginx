// ============================================================================
// 暂不使用: i18n 实体 (改用前端 ts 兜底,后端不再维护 sys_i18n)
// 保留此处仅为方便后续恢复；恢复时取消下方块注释并同步启用 init.sql / 路由 / DTO。
// ============================================================================
/*
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
    pub sort: i32,
    pub version: i32,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub remark: Option<String>,
    #[serde(with = "option_naive_datetime")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "option_naive_datetime")]
    pub updated_at: Option<NaiveDateTime>,
}
*/