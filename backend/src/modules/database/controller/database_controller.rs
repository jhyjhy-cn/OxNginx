use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::ApiResponse;
use crate::modules::common::middleware::TokenInfo;
use crate::modules::database::entity::database::{
    CreateDatabaseRequest, UpdateDatabaseRequest,
};
use crate::modules::database::service::database_service;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub r#type: Option<String>,
}

/// 列表(可按 type 过滤)
pub async fn list_databases(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Json<serde_json::Value> {
    match database_service::list_databases(&state, q.r#type.as_deref()).await {
        Ok(rows) => {
            // ponytail: sqlite 行附 _size_bytes,前端用元数据展示,不存表
            let mut out = Vec::with_capacity(rows.len());
            for d in rows {
                let mut obj = serde_json::to_value(&d).unwrap_or(serde_json::Value::Null);
                if d.r#type == "sqlite" {
                    let p = d.db_name.as_deref().unwrap_or("");
                    let sz = tokio::fs::metadata(p).await.map(|m| m.len() as i64).unwrap_or(0);
                    if let Some(map) = obj.as_object_mut() {
                        map.insert("_size_bytes".into(), serde_json::Value::from(sz));
                    }
                }
                out.push(obj);
            }
            Json(json!(ApiResponse::success(out)))
        }
        Err(e) => Json(json!(ApiResponse::<()>::error(format!(
            "获取数据库连接列表失败: {}",
            e
        )))),
    }
}

/// 详情
pub async fn get_database(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match database_service::get_database(&state, id).await {
        Ok(Some(d)) => Json(json!(ApiResponse::success(d))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("数据库连接不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!(
            "获取数据库连接失败: {}",
            e
        )))),
    }
}

/// 创建
#[audit_log(module = "database", action = "创建数据库连接", capture = req)]
pub async fn create_database(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Extension(info): Extension<TokenInfo>,
    Json(req): Json<CreateDatabaseRequest>,
) -> Json<serde_json::Value> {
    if req.r#type.trim().is_empty() {
        return Json(json!(ApiResponse::<()>::error("类型不能为空")));
    }
    if req.name.trim().is_empty() {
        return Json(json!(ApiResponse::<()>::error("名称不能为空")));
    }
    if !matches!(req.r#type.as_str(), "redis" | "sqlite") {
        return Json(json!(ApiResponse::<()>::error(
            "类型必须是 redis 或 sqlite"
        )));
    }
    match database_service::create_database(&state, req, Some(info.user_id)).await {
        Ok(d) => Json(json!(ApiResponse::success(d))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!(
            "创建数据库连接失败: {}",
            e
        )))),
    }
}

/// 更新(密码空字符串=不修改)
#[audit_log(module = "database", action = "更新数据库连接", capture = req)]
pub async fn update_database(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Extension(info): Extension<TokenInfo>,
    Json(req): Json<UpdateDatabaseRequest>,
) -> Json<serde_json::Value> {
    match database_service::update_database(&state, id, req, Some(info.user_id)).await {
        Ok(Some(d)) => Json(json!(ApiResponse::success(d))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("数据库连接不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!(
            "更新数据库连接失败: {}",
            e
        )))),
    }
}

/// 删除
/// query 参数:
///   remove_file: true 时(sqlite 类型)同时删除 db 文件;默认 false 只删记录
#[audit_log(module = "database", action = "删除数据库连接")]
pub async fn delete_database(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<DeleteQuery>,
) -> Json<serde_json::Value> {
    match database_service::delete_database(&state, id).await {
        Ok(Some(db)) => {
            let mut deleted_file = false;
            if q.remove_file && db.r#type == "sqlite" {
                if let Some(p) = db.db_name.as_deref() {
                    let path = std::path::Path::new(p);
                    if path.exists() {
                        if let Err(e) = tokio::fs::remove_file(path).await {
                            return Json(json!(ApiResponse::<()>::error(format!(
                                "记录已删除,但删除 db 文件失败: {}",
                                e
                            ))));
                        }
                        deleted_file = true;
                    }
                }
            }
            Json(json!(ApiResponse::success(json!({
                "message": "已删除",
                "deleted_file": deleted_file,
            }))))
        }
        Ok(None) => Json(json!(ApiResponse::<()>::error("数据库连接不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!(
            "删除数据库连接失败: {}",
            e
        )))),
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct DeleteQuery {
    #[serde(default)]
    pub remove_file: bool,
}

/// 启停
#[audit_log(module = "database", action = "启停数据库连接")]
pub async fn toggle_database(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match database_service::toggle_database(&state, id).await {
        Ok(Some(d)) => Json(json!(ApiResponse::success(d))),
        Ok(None) => Json(json!(ApiResponse::<()>::error("数据库连接不存在"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!(
            "启停失败: {}",
            e
        )))),
    }
}

/// 探测连通性
#[audit_log(module = "database", action = "测试数据库连接")]
pub async fn test_database(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<serde_json::Value> {
    match database_service::test_connection(&state, id).await {
        Ok(r) => Json(json!(ApiResponse::success(r))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!(
            "探测失败: {}",
            e
        )))),
    }
}
