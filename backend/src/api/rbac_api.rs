use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde_json::json;

use crate::dto::{
    ApiResponse, PageQuery, PagedResult, RbacInfo, ResetPasswordRequest, SetRoleMenusRequest,
    UpsertDeptRequest, UpsertDictItemRequest, UpsertDictRequest, UpsertI18nRequest,
    UpsertMenuRequest, UpsertPostRequest, UpsertRoleRequest, UpsertUserRequest,
};
use crate::middleware::TokenInfo;
use crate::service::rbac_service;
use crate::AppState;

// ============== /api/rbac/me ==============

/// 当前登录用户的 RBAC 信息
pub async fn me(State(state): State<AppState>, Extension(info): Extension<TokenInfo>) -> Json<serde_json::Value> {
    match rbac_service::get_rbac_info(&state.db.pool(), &info.username).await {
        Ok((roles, permissions, menus)) => Json(json!(ApiResponse::success(RbacInfo {
            roles,
            permissions,
            menus,
        }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

// ============== 用户管理 ==============

pub async fn list_users(State(state): State<AppState>, axum::extract::Query(q): axum::extract::Query<PageQuery>) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);
    match rbac_service::list_users_paged(&state.db.pool(), page, page_size, q.keyword.as_deref()).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<UpsertUserRequest>,
) -> Json<serde_json::Value> {
    let password = req.password.unwrap_or_else(|| "123456".into());
    match rbac_service::create_user(&state.db.pool(), &req.username, &password).await {
        Ok(id) => {
            let _ = rbac_service::update_user(
                &state.db.pool(),
                id,
                req.dept_id,
                req.post_id,
                req.disabled,
                req.role_ids,
            )
            .await;
            Json(json!(ApiResponse::success(id)))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertUserRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_user(
        &state.db.pool(),
        id,
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

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_user(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("用户不存在或不可删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn reset_password(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<ResetPasswordRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::reset_password(&state.db.pool(), id, &req.new_password).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

// ============== 角色管理 ==============

pub async fn list_roles(State(state): State<AppState>, axum::extract::Query(q): axum::extract::Query<PageQuery>) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);
    match rbac_service::list_roles_paged(&state.db.pool(), page, page_size, q.keyword.as_deref()).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn create_role(
    State(state): State<AppState>,
    Json(req): Json<UpsertRoleRequest>,
) -> Json<serde_json::Value> {
    let r = rbac_service::create_role(
        &state.db.pool(),
        &req.code,
        &req.name,
        req.remark.as_deref(),
    )
    .await;
    match r {
        Ok(id) => {
            if let Some(mids) = req.menu_ids {
                let _ = rbac_service::set_role_menus(&state.db.pool(), id, &mids).await;
            }
            Json(json!(ApiResponse::success(id)))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn update_role(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertRoleRequest>,
) -> Json<serde_json::Value> {
    let r = rbac_service::update_role(
        &state.db.pool(),
        id,
        Some(req.name.as_str()),
        req.remark.as_deref(),
        req.status.as_deref(),
    )
    .await;
    match r {
        Ok(_) => {
            if let Some(mids) = req.menu_ids {
                let _ = rbac_service::set_role_menus(&state.db.pool(), id, &mids).await;
            }
            Json(json!(ApiResponse::success("ok")))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn delete_role(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_role(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("角色不存在或不可删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn set_role_menus(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<SetRoleMenusRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::set_role_menus(&state.db.pool(), id, &req.menu_ids).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

// ============== 部门管理 ==============

pub async fn list_depts(State(state): State<AppState>, axum::extract::Query(q): axum::extract::Query<PageQuery>) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(100).max(1);
    match rbac_service::list_depts_paged(&state.db.pool(), page, page_size, q.keyword.as_deref()).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn create_dept(
    State(state): State<AppState>,
    Json(req): Json<UpsertDeptRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_dept(
        &state.db.pool(),
        &req.name,
        req.parent_id,
        req.sort.unwrap_or(0),
    )
    .await
    {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn update_dept(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertDeptRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_dept(
        &state.db.pool(),
        id,
        Some(req.name.as_str()),
        Some(req.parent_id),
        req.sort,
    )
    .await
    {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn delete_dept(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_dept(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("部门不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

// ============== 岗位管理 ==============

pub async fn list_posts(State(state): State<AppState>, axum::extract::Query(q): axum::extract::Query<PageQuery>) -> Json<serde_json::Value> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).max(1);
    match rbac_service::list_posts_paged(&state.db.pool(), page, page_size, q.keyword.as_deref()).await {
        Ok((list, total)) => Json(json!(ApiResponse::success(PagedResult { list, total, page, page_size }))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn create_post(
    State(state): State<AppState>,
    Json(req): Json<UpsertPostRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_post(
        &state.db.pool(),
        &req.code,
        &req.name,
        req.sort.unwrap_or(0),
    )
    .await
    {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertPostRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_post(
        &state.db.pool(),
        id,
        Some(req.name.as_str()),
        req.sort,
    )
    .await
    {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_post(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("岗位不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

// ============== 菜单管理 ==============

pub async fn list_menus(State(state): State<AppState>) -> Json<serde_json::Value> {
    match rbac_service::list_menus(&state.db.pool()).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn create_menu(
    State(state): State<AppState>,
    Json(req): Json<UpsertMenuRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_menu(
        &state.db.pool(),
        &req.name,
        &req.title,
        req.parent_id,
        req.icon.as_deref(),
        req.path.as_deref(),
        req.component.as_deref(),
        &req.menu_type,
        req.permission.as_deref(),
        req.sort.unwrap_or(0),
    )
    .await
    {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn update_menu(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertMenuRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_menu(
        &state.db.pool(),
        id,
        Some(req.name.as_str()),
        Some(req.title.as_str()),
        Some(req.parent_id),
        Some(req.icon.as_deref()),
        Some(req.path.as_deref()),
        Some(req.component.as_deref()),
        Some(req.permission.as_deref()),
        req.sort,
    )
    .await
    {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn delete_menu(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_menu(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("菜单不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn batch_delete_menus(
    State(state): State<AppState>,
    Json(ids): Json<Vec<i64>>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_menus(&state.db.pool(), &ids).await {
        Ok(n) => Json(json!(ApiResponse::success(format!("已删除 {} 条", n)))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

// ============== 国际化管理 ==============

pub async fn list_i18n_locales(State(state): State<AppState>) -> Json<serde_json::Value> {
    match rbac_service::list_i18n_locales(&state.db.pool()).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn list_i18n(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    // 分页模式：有 page 参数走分页，否则走全量（兼容 fetchI18n 全量拉取）
    if let Some(page_str) = params.get("page") {
        let page: i64 = page_str.parse().unwrap_or(1);
        let page_size: i64 = params.get("page_size").and_then(|s| s.parse().ok()).unwrap_or(100);
        let key = params.get("key").map(|s| s.as_str());
        match rbac_service::list_i18n_paged(&state.db.pool(), page.max(1), page_size.max(1), key).await {
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
        match rbac_service::list_i18n(&state.db.pool(), locale).await {
            Ok(data) => Json(json!(ApiResponse::success(data))),
            Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
        }
    }
}

pub async fn upsert_i18n(
    State(state): State<AppState>,
    Json(req): Json<UpsertI18nRequest>,
) -> Json<serde_json::Value> {
    let pairs: Vec<(String, String)> = req.entries.into_iter().map(|e| (e.key, e.value)).collect();
    match rbac_service::upsert_i18n_batch(&state.db.pool(), &req.locale, &pairs).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn delete_i18n(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match rbac_service::delete_i18n(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("记录不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn get_i18n_messages(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let locale = params.get("locale").map(|s| s.as_str()).unwrap_or("zh-CN");
    match rbac_service::get_i18n_messages(&state.db.pool(), locale).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

// ============== 字典管理 ==============

pub async fn list_dicts(State(state): State<AppState>) -> Json<serde_json::Value> {
    match rbac_service::list_dicts(&state.db.pool()).await {
        Ok(data) => Json(json!(ApiResponse::success(data))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn get_dict(State(state): State<AppState>, Path(id): Path<i64>) -> Json<serde_json::Value> {
    match rbac_service::get_dict_with_items(&state.db.pool(), id).await {
        Ok(Some(data)) => Json(json!(ApiResponse::success(data))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("字典不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn create_dict(
    State(state): State<AppState>,
    Json(req): Json<UpsertDictRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_dict(&state.db.pool(), &req.name, &req.code, req.description.as_deref()).await {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn update_dict(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertDictRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_dict(&state.db.pool(), id, Some(&req.name), req.description.as_deref(), None).await {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn delete_dict(State(state): State<AppState>, Path(id): Path<i64>) -> Json<serde_json::Value> {
    match rbac_service::delete_dict(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("字典不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn create_dict_item(
    State(state): State<AppState>,
    Path(dict_id): Path<i64>,
    Json(req): Json<UpsertDictItemRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::create_dict_item(
        &state.db.pool(),
        dict_id,
        &req.label,
        &req.value,
        req.sort.unwrap_or(0),
    )
    .await
    {
        Ok(id) => Json(json!(ApiResponse::success(id))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn update_dict_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpsertDictItemRequest>,
) -> Json<serde_json::Value> {
    match rbac_service::update_dict_item(
        &state.db.pool(),
        id,
        Some(&req.label),
        Some(&req.value),
        req.sort,
        req.status.as_deref(),
    )
    .await
    {
        Ok(_) => Json(json!(ApiResponse::success("ok"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}

pub async fn delete_dict_item(State(state): State<AppState>, Path(id): Path<i64>) -> Json<serde_json::Value> {
    match rbac_service::delete_dict_item(&state.db.pool(), id).await {
        Ok(true) => Json(json!(ApiResponse::success("ok"))),
        Ok(false) => Json(json!(ApiResponse::<()>::error("字典项不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(e.to_string()))),
    }
}