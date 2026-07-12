use axum::{extract::{Extension, Path, State}, Json};
use serde_json::json;

use ox_nginx_macros::{audit_log, check_permission};

use crate::modules::common::dto::{ApiResponse, UpsertI18nRequest};
use crate::modules::common::middleware::TokenInfo;
use crate::modules::sys::service::i18n_service;
use crate::AppState;

// ============== 国际化管理 =============

#[check_permission("sys:i18n:query")]
pub async fn list_i18n_locales(State(state): State<AppState>, token: Extension<TokenInfo>) -> Json<serde_json::Value> {
    match i18n_service::list_i18n_locales(&state.db.pool()).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:i18n:query")]
pub async fn list_i18n(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    token: Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    if let Some(page_str) = params.get("page") {
        let page: i64 = page_str.parse().unwrap_or(1);
        let page_size: i64 = params.get("page_size").and_then(|s| s.parse().ok()).unwrap_or(100);
        let key = params.get("key").map(|s| s.as_str());
        match i18n_service::list_i18n_paged(&state.db.pool(), page.max(1), page_size.max(1), key).await {
            Ok((data, total)) => Json(json!(ApiResponse::success(serde_json::json!({
                "list": data,
                "total": total,
                "page": page,
                "page_size": page_size,
            })))),
            Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
        }
    } else {
        let locale = params.get("locale").map(|s| s.as_str());
        match i18n_service::list_i18n(&state.db.pool(), locale).await {
            Ok(data) => Json(json!(ApiResponse::success(data))),
            Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
        }
    }
}

#[check_permission("sys:i18n:edit")]
#[audit_log(module = "rbac", action = "保存国际化", capture = req)]
pub async fn upsert_i18n(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<UpsertI18nRequest>,
) -> Json<serde_json::Value> {
    let pairs: Vec<(String, String)> = req.entries.into_iter().map(|e| (e.key, e.value)).collect();
    match i18n_service::upsert_i18n_batch(&state.db.pool(), &req.locale, &pairs).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:i18n:delete")]
#[audit_log(module = "rbac", action = "删除国际化")]
pub async fn delete_i18n(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match i18n_service::delete_i18n(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("记录不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

/// 获取 i18n 消息（任意登录用户可用，从 rbac_controller 拆分过来）
pub async fn get_i18n_messages(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let locale = params.get("locale").map(|s| s.as_str()).unwrap_or("zh-CN");
    match i18n_service::get_i18n_messages(&state.db.pool(), locale).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}
