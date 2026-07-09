use axum::{extract::State, Json};
use serde_json::json;

use crate::modules::common::dto::ApiResponse;
use crate::modules::site::service::cert_service;
use crate::modules::site::dao::cert_dao;
use crate::AppState;

/// 获取证书列表
pub async fn list_certificates(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match cert_service::get_all_certs(&state).await {
        Ok(certs) => Json(json!(ApiResponse::success(certs))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("获取证书列表失败: {}", e)))),
    }
}

/// 申请证书
pub async fn apply_certificate(
    State(state): State<AppState>,
    Json(req): Json<crate::modules::common::dto::ApplyCertRequest>,
) -> Json<serde_json::Value> {
    match cert_service::apply_cert(&state, &req.domain).await {
        Ok(cert) => Json(json!(ApiResponse::success(cert))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("申请证书失败: {}", e)))),
    }
}

/// 续期证书
pub async fn renew_certificate(
    State(state): State<AppState>,
    Json(req): Json<crate::modules::common::dto::ApplyCertRequest>,
) -> Json<serde_json::Value> {
    let cert = cert_dao::find_cert_by_domain(state.db.pool(), &req.domain).await;

    match cert {
        Ok(Some(c)) => match cert_service::renew_cert(&state, c.id).await {
            Ok(true) => Json(json!(ApiResponse::success("证书续期成功"))),
            Ok(false) => Json(json!(ApiResponse::<()>::error("证书续期失败"))),
            Err(e) => Json(json!(ApiResponse::<()>::error(format!("证书续期失败: {}", e)))),
        },
        Ok(None) => Json(json!(ApiResponse::<()>::error("证书不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("查询证书失败: {}", e)))),
    }
}