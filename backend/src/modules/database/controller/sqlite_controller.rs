use axum::{
    extract::{Json, State},
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use ox_nginx_macros::audit_log;

use crate::modules::common::dto::ApiResponse;
use crate::modules::common::middleware::TokenInfo;
use crate::modules::database::service::sqlite_service;
use crate::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct PathQuery {
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TablePageQuery {
    pub path: String,
    pub table: String,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}
fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    20
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InsertBody {
    pub path: String,
    pub table: String,
    pub values: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateBody {
    pub path: String,
    pub table: String,
    pub pk: serde_json::Map<String, serde_json::Value>,
    pub values: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteBody {
    pub path: String,
    pub table: String,
    pub pk: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExecBody {
    pub path: String,
    pub sql: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTableBody {
    pub path: String,
    pub sql: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RenameTableBody {
    pub path: String,
    pub old: String,
    pub new: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DropTableBody {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddColumnBody {
    pub path: String,
    pub table: String,
    pub col_def: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RenameColumnBody {
    pub path: String,
    pub table: String,
    pub old: String,
    pub new: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DropColumnBody {
    pub path: String,
    pub table: String,
    pub col: String,
}

fn to_path(p: &str) -> Result<std::path::PathBuf, String> {
    sqlite_service::resolve_db_path(p).map_err(|e| e.to_string())
}

/// 列表数据库内所有表
pub async fn list_tables(
    State(_state): State<AppState>,
    Json(q): Json<PathQuery>,
) -> Json<serde_json::Value> {
    let p = match to_path(&q.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::list_tables(&p).await {
        Ok(rows) => Json(json!(ApiResponse::success(rows))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("列出表失败: {}", e)))),
    }
}

/// 拉表数据(分页)
pub async fn table_data(
    State(_state): State<AppState>,
    Json(q): Json<TablePageQuery>,
) -> Json<serde_json::Value> {
    let p = match to_path(&q.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::table_data(&p, &q.table, q.page, q.page_size).await {
        Ok(d) => Json(json!(ApiResponse::success(d))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("查询表数据失败: {}", e)))),
    }
}

/// 插入一行
#[audit_log(module = "database", action = "SQLite 插入行")]
pub async fn row_insert(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<InsertBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::row_insert(&p, &body.table, &body.values).await {
        Ok(id) => Json(json!(ApiResponse::success(json!({ "id": id })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("插入失败: {}", e)))),
    }
}

/// 更新一行
#[audit_log(module = "database", action = "SQLite 更新行")]
pub async fn row_update(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<UpdateBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::row_update(&p, &body.table, &body.pk, &body.values).await {
        Ok(n) => Json(json!(ApiResponse::success(json!({ "affected": n })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("更新失败: {}", e)))),
    }
}

/// 删除一行
#[audit_log(module = "database", action = "SQLite 删除行")]
pub async fn row_delete(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<DeleteBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::row_delete(&p, &body.table, &body.pk).await {
        Ok(n) => Json(json!(ApiResponse::success(json!({ "affected": n })))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除失败: {}", e)))),
    }
}

/// 任意只读 SQL
#[audit_log(module = "database", action = "SQLite 执行查询", capture = body)]
pub async fn exec_sql(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<ExecBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::exec_sql(&p, &body.sql).await {
        Ok(r) => Json(json!(ApiResponse::success(r))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("执行失败: {}", e)))),
    }
}

/// 创建表
#[audit_log(module = "database", action = "SQLite 创建表", capture = body)]
pub async fn create_table(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<CreateTableBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::create_table(&p, &body.sql).await {
        Ok(_) => Json(json!(ApiResponse::success("已创建"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("创建表失败: {}", e)))),
    }
}

/// 重命名表
#[audit_log(module = "database", action = "SQLite 重命名表", capture = body)]
pub async fn rename_table(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<RenameTableBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::rename_table(&p, &body.old, &body.new).await {
        Ok(_) => Json(json!(ApiResponse::success("已重命名"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("重命名失败: {}", e)))),
    }
}

/// 删除表
#[audit_log(module = "database", action = "SQLite 删除表", capture = body)]
pub async fn drop_table(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<DropTableBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::drop_table(&p, &body.name).await {
        Ok(_) => Json(json!(ApiResponse::success("已删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除表失败: {}", e)))),
    }
}

/// 添加列
#[audit_log(module = "database", action = "SQLite 添加列", capture = body)]
pub async fn add_column(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<AddColumnBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::add_column(&p, &body.table, &body.col_def).await {
        Ok(_) => Json(json!(ApiResponse::success("已添加"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("添加列失败: {}", e)))),
    }
}

/// 重命名列
#[audit_log(module = "database", action = "SQLite 重命名列", capture = body)]
pub async fn rename_column(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<RenameColumnBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::rename_column(&p, &body.table, &body.old, &body.new).await {
        Ok(_) => Json(json!(ApiResponse::success("已重命名"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("重命名列失败: {}", e)))),
    }
}

/// 删除列
#[audit_log(module = "database", action = "SQLite 删除列", capture = body)]
pub async fn drop_column(
    ctx: Extension<crate::modules::common::audit::context::SharedAuditContext>,
    State(_state): State<AppState>,
    Extension(_info): Extension<TokenInfo>,
    Json(body): Json<DropColumnBody>,
) -> Json<serde_json::Value> {
    let p = match to_path(&body.path) {
        Ok(p) => p,
        Err(e) => return Json(json!(ApiResponse::<()>::error(e))),
    };
    match sqlite_service::drop_column(&p, &body.table, &body.col).await {
        Ok(_) => Json(json!(ApiResponse::success("已删除"))),
        Err(e) => Json(json!(ApiResponse::<()>::error(format!("删除列失败: {}", e)))),
    }
}
