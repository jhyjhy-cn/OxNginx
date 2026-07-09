use axum::{extract::{Path, State}, Json};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::dto::{ApiResponse, PagedResult, ResetPasswordRequest, UpsertUserRequest, UserQuery};
use crate::service::rbac_service;
use crate::AppState;

// ============== 用户管理 ==============

pub async fn list_users(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<UserQuery>,
) -> Json<serde_json::Value> {
    match rbac_service::list_users_paged(&state.db.pool(), &q).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult {
            list,
            total,
            page: q.page.unwrap_or(1),
            page_size: q.page_size.unwrap_or(20),
        }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::get_user(&state.db.pool(), id).await {
        Ok(Some(user)) => {
            let role_ids = rbac_service::get_user_role_ids(&state.db.pool(), id).await.unwrap_or_default();
            Json(json!(ApiResponse::success(serde_json::json!({
                "id": user.id,
                "username": user.username,
                "nickname": user.nickname,
                "phone": user.phone,
                "email": user.email,
                "gender": user.gender,
                "remark": user.remark,
                "dept_id": user.dept_id,
                "post_id": user.post_id,
                "disabled": user.disabled,
                "role_ids": role_ids,
            }))))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("用户不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "创建用户", capture = req)]
pub async fn create_user(
    ctx: axum::extract::Extension<crate::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<UpsertUserRequest>,
) -> Json<serde_json::Value> {
    let password = req.password.unwrap_or_else(|| "123456".into());
    match rbac_service::create_user(&state.db.pool(), &req.username, &password).await {
        Ok(id) => {
            let _ = rbac_service::update_user(
                &state.db.pool(),
                id,
                req.nickname.as_deref(),
                req.phone.as_deref(),
                req.email.as_deref(),
                req.gender.as_deref(),
                req.remark.as_deref(),
                req.dept_id,
                req.post_id,
                req.disabled.or(Some(0)),
                req.role_ids,
            )
            .await;
            Json(json!(ApiResponse::success(id)))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "更新用户", capture = req)]
pub async fn update_user(
    ctx: axum::extract::Extension<crate::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertUserRequest>,
) -> Json<serde_json::Value> {
    // 禁止禁用/删除 admin 用户
    if let Some(dis) = req.disabled {
        if dis == 1 && rbac_service::is_admin_user(&state.db.pool(), id).await.unwrap_or(false) {
            return Json(json!(ApiResponse::<()>::error("禁止禁用管理员账户")));
        }
    }

    match rbac_service::update_user(
        &state.db.pool(),
        id,
        req.nickname.as_deref(),
        req.phone.as_deref(),
        req.email.as_deref(),
        req.gender.as_deref(),
        req.remark.as_deref(),
        req.dept_id,
        req.post_id,
        req.disabled,
        req.role_ids,
    )
    .await
    {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "删除用户")]
pub async fn delete_user(
    ctx: axum::extract::Extension<crate::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_user(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("用户不存在或不可删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[audit_log(module = "rbac", action = "重置密码", capture = req)]
pub async fn reset_password(
    ctx: axum::extract::Extension<crate::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<ResetPasswordRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::reset_password(&state.db.pool(), id, &req.new_password).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}
