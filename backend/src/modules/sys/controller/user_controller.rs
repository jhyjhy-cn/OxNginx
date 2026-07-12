use axum::{extract::{Extension, Path, State}, Json};
use axum::response::Response;
use serde_json::json;

use ox_nginx_macros::{audit_log, check_permission};

use crate::modules::common::dto::{ApiResponse, PagedResult, ResetPasswordRequest, RbacInfo, UpsertUserRequest, UserQuery};
use crate::modules::common::middleware::TokenInfo;
use crate::modules::sys::service::user_service;
use crate::AppState;

// ============== 用户管理 ==============

#[check_permission("sys:user:query")]
pub async fn list_users(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<UserQuery>,
    token: Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    match user_service::list_users_paged(&state.db.pool(), &q).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult {
            list,
            total,
            page: q.page.unwrap_or(1),
            page_size: q.page_size.unwrap_or(20),
        }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:user:query")]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    token: Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    match user_service::get_user(&state.db.pool(), id).await {
        Ok(Some(user)) => {
            let role_ids = user_service::get_user_role_ids(&state.db.pool(), id).await.unwrap_or_default();
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

#[check_permission("sys:user:add")]
#[audit_log(module = "rbac", action = "创建用户", capture = req)]
pub async fn create_user(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<UpsertUserRequest>,
) -> Json<serde_json::Value> {
    let password = req.password.unwrap_or_else(|| "123456".into());
    match user_service::create_user(&state.db.pool(), &req.username, &password).await {
        Ok(id) => {
            let _ = user_service::update_user(
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

#[check_permission("sys:user:edit")]
#[audit_log(module = "rbac", action = "更新用户", capture = req)]
pub async fn update_user(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertUserRequest>,
) -> Json<serde_json::Value> {
    // 禁止禁用/删除 admin 用户
    if let Some(dis) = req.disabled {
        if dis == 1 && user_service::is_admin_user(&state.db.pool(), id).await.unwrap_or(false) {
            return Json(json!(ApiResponse::<()>::error("禁止禁用管理员账户")));
        }
    }

    match user_service::update_user(
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

#[check_permission("sys:user:delete")]
#[audit_log(module = "rbac", action = "删除用户")]
pub async fn delete_user(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match user_service::delete_user(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("用户不存在或不可删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:user:resetPwd")]
#[audit_log(module = "rbac", action = "重置密码", capture = req)]
pub async fn reset_password(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<ResetPasswordRequest>,
) -> Json<serde_json::Value> {
    match user_service::reset_password(&state.db.pool(), id, &req.new_password).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

/// 当前登录用户的 RBAC 信息（从 rbac_controller 拆分过来）
pub async fn me(
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
) -> Json<serde_json::Value> {
    match user_service::get_rbac_info(&state.db.pool(), &info.username).await {
        Ok((roles, permissions, menus)) => {
            Json(json!(ApiResponse::success(RbacInfo { roles, permissions, menus })))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BatchResetPasswordRequest {
    pub ids: Vec<i64>,
    #[serde(default = "default_password")]
    pub new_password: String,
}

fn default_password() -> String {
    "123456".to_string()
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BatchDisabledRequest {
    pub ids: Vec<i64>,
    pub disabled: i32,
}

#[check_permission("sys:user:batchResetPwd")]
#[audit_log(module = "rbac", action = "批量重置密码", capture = req)]
pub async fn batch_reset_password(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<BatchResetPasswordRequest>,
) -> Json<serde_json::Value> {
    if req.ids.is_empty() {
        return Json(json!(ApiResponse::<()>::error("请选择要重置的用户")));
    }
    match user_service::batch_reset_password(&state.db.pool(), &req.ids, &req.new_password).await {
        Ok(n) => Json(json!(ApiResponse::success(format!("已重置 {} 个用户密码", n)))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

#[check_permission("sys:user:changeStatus")]
#[audit_log(module = "rbac", action = "批量禁用/启用用户", capture = req)]
pub async fn batch_set_disabled(
    token: Extension<TokenInfo>,
    ctx: axum::extract::Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Json(req): Json<BatchDisabledRequest>,
) -> Json<serde_json::Value> {
    if req.ids.is_empty() {
        return Json(json!(ApiResponse::<()>::error("请选择要操作的用户")));
    }
    match user_service::batch_set_disabled(&state.db.pool(), &req.ids, req.disabled).await {
        Ok(n) => {
            let action = if req.disabled == 1 { "禁用" } else { "启用" };
            Json(json!(ApiResponse::success(format!("已{} {} 个用户", action, n))))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

/// 按当前查询条件导出用户为 xlsx
pub async fn export_users(
    State(state): State<AppState>,
    _token: Extension<TokenInfo>,
    axum::extract::Query(q): axum::extract::Query<crate::modules::common::dto::UserQuery>,
) -> Response {
    use crate::modules::common::util::excel::{build_xlsx, export_error, xlsx_response, Sheet};

    let list = match user_service::list_users_for_export(&state.db.pool(), &q).await {
        Ok(v) => v,
        Err(e) => return export_error(e),
    };

    let headers = vec![
        "ID".into(),
        "用户名".into(),
        "昵称".into(),
        "部门".into(),
        "岗位".into(),
        "手机".into(),
        "邮箱".into(),
        "性别".into(),
        "状态".into(),
        "创建时间".into(),
    ];
    let rows: Vec<Vec<String>> = list
        .iter()
        .map(|u| {
            vec![
                u.id.to_string(),
                u.username.clone(),
                u.nickname.clone().unwrap_or_default(),
                u.dept_name.clone().unwrap_or_default(),
                u.post_name.clone().unwrap_or_default(),
                u.phone.clone().unwrap_or_default(),
                u.email.clone().unwrap_or_default(),
                u.gender.clone().unwrap_or_default(),
                (if u.disabled == 1 { "禁用" } else { "启用" }).into(),
                u.created_at
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
            ]
        })
        .collect();

    let sheet = Sheet { headers, rows };
    let filename = format!("users-{}.xlsx", chrono::Local::now().format("%Y%m%d%H%M%S"));
    match build_xlsx("Users", &sheet) {
        Ok(buf) => xlsx_response(filename, buf),
        Err(e) => export_error(e),
    }
}