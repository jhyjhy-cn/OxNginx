use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::modules::common::dto::{ApiResponse, UpsertParamRequest};
use crate::modules::sys::service::param_service as svc;
use crate::AppState;

// ============== 系统参数 =============

#[derive(Debug, Deserialize)]
pub struct PageParamsQuery {
    pub keyword: Option<String>,
    pub group_code: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn page_params(
    State(state): State<AppState>,
    Query(q): Query<PageParamsQuery>,
) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    match svc::page_params(
        &state.db.pool(),
        q.keyword.as_deref(),
        q.group_code.as_deref(),
        page,
        page_size,
    )
    .await
    {
        Ok((list, total)) => Json(json!(ApiResponse::success(serde_json::json!({
            "list": list,
            "total": total,
            "page": page,
            "page_size": page_size,
        })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn get_param(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match svc::get_param(&state.db.pool(), id).await {
        Ok(Some(p)) => Json(json!(ApiResponse::success(p))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("参数不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn get_param_by_key(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Json<serde_json::Value> {
    match svc::get_param_by_key(&state.db.pool(), &key).await {
        Ok(Some(p)) => Json(json!(ApiResponse::success(p))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("参数不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn create_param(
    State(state): State<AppState>,
    Json(req): Json<UpsertParamRequest>,
) -> Json<serde_json::Value> {
    match svc::create_param(
        &state.db.pool(),
        &req.key,
        req.value.as_deref(),
        &req.name,
        req.group_code.as_deref(),
        req.remark.as_deref(),
        req.sort.unwrap_or(0),
    )
    .await
    {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn update_param(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertParamRequest>,
) -> Json<serde_json::Value> {
    match svc::update_param(
        &state.db.pool(),
        id,
        req.value.as_deref(),
        Some(&req.name),
        req.group_code.as_deref(),
        req.remark.as_deref(),
        req.sort,
    )
    .await
    {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn delete_param(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match svc::delete_param(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("参数不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}