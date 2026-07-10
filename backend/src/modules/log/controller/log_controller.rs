use axum::{extract::State, http::header, Json};
use serde_json::json;

use crate::modules::common::dto::{ApiResponse, LogResponse, PagedResult};
use crate::modules::common::enums::LogStatus;
use crate::modules::log::service::log_service::{LoginLogQuery, OperationLogQuery};
use crate::modules::common::util::read_log_tail;
use crate::AppState;

/// ponytail: 前端发 success/failed 字符串,这里转成 LogStatus::as_i32(),空值 / 其它字符串 -> None
fn parse_status_str(s: Option<String>) -> Option<i32> {
    match s.as_deref().filter(|v| !v.is_empty())? {
        "success" => Some(LogStatus::Success.as_i32()),
        "failed" => Some(LogStatus::Failed.as_i32()),
        _ => None,
    }
}

/// 获取Access日志
pub async fn access_log(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = state.get_config();
    let log_path = &config.nginx.log_access;
    match read_log_tail(log_path, 100).await {
        Ok(lines) => Json(json!(ApiResponse::success(LogResponse { lines }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取日志失败: {}", e)))),
    }
}

/// 获取Error日志
pub async fn error_log(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = state.get_config();
    let log_path = &config.nginx.log_error;
    match read_log_tail(log_path, 100).await {
        Ok(lines) => Json(json!(ApiResponse::success(LogResponse { lines }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("读取日志失败: {}", e)))),
    }
}

#[derive(serde::Deserialize)]
pub struct OperationLogParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub username: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub module: Option<String>,
    pub trace_id: Option<String>,
}

fn build_op_query(p: &OperationLogParams) -> OperationLogQuery {
    OperationLogQuery {
        page: p.page.unwrap_or(1).max(1),
        page_size: p.page_size.unwrap_or(20).max(1),
        username: p.username.clone().filter(|s| !s.is_empty()),
        status: parse_status_str(p.status.clone()),
        start_time: p.start_time.clone().filter(|s| !s.is_empty()),
        end_time: p.end_time.clone().filter(|s| !s.is_empty()),
        module: p.module.clone().filter(|s| !s.is_empty()),
        trace_id: p.trace_id.clone().filter(|s| !s.is_empty()),
    }
}

/// 操作日志列表
pub async fn list_operation_logs(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<OperationLogParams>,
) -> Json<serde_json::Value> {
    let q = build_op_query(&params);
    let page = q.page;
    let page_size = q.page_size;
    match crate::modules::log::service::log_service::list_operation_logs(state.db.pool(), &q).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

/// 导出操作日志 CSV
pub async fn export_operation_logs(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<OperationLogParams>,
) -> Result<axum::response::Response, Json<serde_json::Value>> {
    let q = build_op_query(&params);
    match crate::modules::log::service::log_service::export_operation_logs_csv(state.db.pool(), &q).await {
        Ok(csv) => {
            let response = axum::response::Response::builder()
                .status(200)
                .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"operation_logs.csv\"")
                .body(axum::body::Body::from(csv))
                .unwrap();
            Ok(response)
        }
        Err(e) => Err(Json(json!(ApiResponse::<()>::error(e.to_string())))),
    }
}

#[derive(serde::Deserialize)]
pub struct LoginLogParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

fn build_query(params: &LoginLogParams) -> LoginLogQuery {
    LoginLogQuery {
        page: params.page.unwrap_or(1).max(1),
        page_size: params.page_size.unwrap_or(20).max(1),
        username: params.username.clone().filter(|s| !s.is_empty()),
        ip: params.ip.clone().filter(|s| !s.is_empty()),
        status: parse_status_str(params.status.clone()),
        start_time: params.start_time.clone().filter(|s| !s.is_empty()),
        end_time: params.end_time.clone().filter(|s| !s.is_empty()),
    }
}

/// 登录日志列表
pub async fn list_login_logs(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<LoginLogParams>,
) -> Json<serde_json::Value> {
    let q = build_query(&params);
    let page = q.page;
    let page_size = q.page_size;
    match crate::modules::log::service::log_service::list_login_logs(state.db.pool(), &q).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

/// 导出登录日志 CSV
pub async fn export_login_logs(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<LoginLogParams>,
) -> Result<axum::response::Response, Json<serde_json::Value>> {
    let q = build_query(&params);
    match crate::modules::log::service::log_service::export_login_logs_csv(state.db.pool(), &q).await {
        Ok(csv) => {
            let response = axum::response::Response::builder()
                .status(200)
                .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"login_logs.csv\"")
                .body(axum::body::Body::from(csv))
                .unwrap();
            Ok(response)
        }
        Err(e) => Err(Json(json!(ApiResponse::<()>::error(e.to_string())))),
    }
}
