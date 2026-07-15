use sqlx::SqlitePool;

use crate::modules::database::entity::database::{CreateDatabaseRequest, Database};

/// 列表(全部)
pub async fn list_all(pool: &SqlitePool) -> sqlx::Result<Vec<Database>> {
    sqlx::query_as::<_, Database>(
        "SELECT id, type, name, host, port, username, password, db_name, db_path,
                enabled, sort, remark, version, created_by, updated_by, created_at, updated_at
         FROM dbm_databases ORDER BY sort ASC, id ASC",
    )
    .fetch_all(pool)
    .await
}

/// 按类型过滤
pub async fn list_by_type(pool: &SqlitePool, db_type: &str) -> sqlx::Result<Vec<Database>> {
    sqlx::query_as::<_, Database>(
        "SELECT id, type, name, host, port, username, password, db_name, db_path,
                enabled, sort, remark, version, created_by, updated_by, created_at, updated_at
         FROM dbm_databases WHERE type = ? ORDER BY sort ASC, id ASC",
    )
    .bind(db_type)
    .fetch_all(pool)
    .await
}

/// 按 id 查
pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Database>> {
    sqlx::query_as::<_, Database>(
        "SELECT id, type, name, host, port, username, password, db_name, db_path,
                enabled, sort, remark, version, created_by, updated_by, created_at, updated_at
         FROM dbm_databases WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// 按 name 查(用于 name 唯一性预检)
pub async fn find_by_name(pool: &SqlitePool, name: &str) -> sqlx::Result<Option<Database>> {
    sqlx::query_as::<_, Database>(
        "SELECT id, type, name, host, port, username, password, db_name, db_path,
                enabled, sort, remark, version, created_by, updated_by, created_at, updated_at
         FROM dbm_databases WHERE name = ?",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

/// 插入
pub async fn insert(
    pool: &SqlitePool,
    req: &CreateDatabaseRequest,
    created_by: Option<i64>,
) -> sqlx::Result<Database> {
    sqlx::query_as::<_, Database>(
        r#"INSERT INTO dbm_databases
           (type, name, host, port, username, password, db_name, db_path, enabled, sort, remark, created_by)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING id, type, name, host, port, username, password, db_name, db_path,
                     enabled, sort, remark, version, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(&req.r#type)
    .bind(&req.name)
    .bind(&req.host)
    .bind(req.port)
    .bind(&req.username)
    .bind(&req.password)
    .bind(&req.db_name)
    .bind(&req.db_path)
    .bind(if req.enabled.unwrap_or(true) { 1 } else { 0 })
    .bind(req.sort.unwrap_or(0))
    .bind(&req.remark)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

/// 更新:password=None 表示不动该字段
#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    r#type: &str,
    name: &str,
    host: Option<&String>,
    port: Option<i64>,
    username: Option<&String>,
    password: Option<&str>,
    db_name: Option<&String>,
    db_path: Option<&String>,
    enabled: i32,
    sort: i32,
    remark: Option<&String>,
    updated_by: Option<i64>,
) -> sqlx::Result<Option<Database>> {
    let result = sqlx::query_as::<_, Database>(
        r#"UPDATE dbm_databases
           SET type=?, name=?, host=?, port=?, username=?, password=?, db_name=?, db_path=?,
               enabled=?, sort=?, remark=?, updated_by=?, version=version+1, updated_at=CURRENT_TIMESTAMP
           WHERE id=?
           RETURNING id, type, name, host, port, username, password, db_name, db_path,
                     enabled, sort, remark, version, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(r#type)
    .bind(name)
    .bind(host)
    .bind(port)
    .bind(username)
    .bind(password)
    .bind(db_name)
    .bind(db_path)
    .bind(enabled)
    .bind(sort)
    .bind(remark)
    .bind(updated_by)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(result)
}

/// 启停(翻转 enabled)
pub async fn toggle(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Database>> {
    sqlx::query_as::<_, Database>(
        r#"UPDATE dbm_databases
           SET enabled = 1 - enabled, version=version+1, updated_at=CURRENT_TIMESTAMP
           WHERE id=?
           RETURNING id, type, name, host, port, username, password, db_name, db_path,
                     enabled, sort, remark, version, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// 删除
pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM dbm_databases WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}
