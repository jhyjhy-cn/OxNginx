use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OperationLog {
    pub id: i64,
    pub trace_id: Option<String>,
    pub username: String,
    pub module: Option<String>,
    pub action: String,
    pub method: Option<String>,
    pub uri: Option<String>,
    pub ip: Option<String>,
    pub status: String,
    pub cost_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_msg: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LoginLog {
    pub id: i64,
    pub username: String,
    pub ip: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub log_type: String,
    pub status: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}