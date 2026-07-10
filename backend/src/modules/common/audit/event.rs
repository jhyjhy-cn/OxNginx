use chrono::{NaiveDateTime, Utc};

#[derive(Debug, serde::Serialize)]
pub struct AuditEvent {
    pub trace_id: String,
    pub username: String,
    pub module: String,
    pub action: String,
    pub method: String,
    pub uri: String,
    pub ip: Option<String>,
    pub status: i32, // 1=启用 0=禁用
    pub duration_ms: i64,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_msg: Option<String>,
    #[serde(serialize_with = "crate::modules::common::util::datetime::naive_datetime::serialize")]
    pub created_at: NaiveDateTime,
}

impl AuditEvent {
    pub fn now(trace_id: String) -> Self {
        Self {
            trace_id,
            username: String::new(),
            module: String::new(),
            action: String::new(),
            method: String::new(),
            uri: String::new(),
            ip: None,
            status: 1,
            duration_ms: 0,
            request_body: None,
            response_body: None,
            error_msg: None,
            created_at: Utc::now().naive_utc(),
        }
    }
}
