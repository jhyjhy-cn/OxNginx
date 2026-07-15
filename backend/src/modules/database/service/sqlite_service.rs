use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::{Column, Row, SqlitePool};

use crate::modules::common::config::get_run_dir;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

#[allow(dead_code)]
const DEFAULT_DIR: &str = "server/sqlite";

/// 列信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub r#type: String,
    pub notnull: bool,
    pub pk: bool,
    pub default_value: Option<String>,
}

/// 表信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub row_count: i64,
    pub size_bytes: i64,
}

/// 表格分页数据
#[derive(Debug, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<ColumnInfo>,
    pub primary_key: Vec<String>,
    pub rows: Vec<serde_json::Map<String, serde_json::Value>>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// SQL 查询结果(只读)
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlResult {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Map<String, serde_json::Value>>,
    pub rows_affected: i64,
    pub error: Option<String>,
}

pub fn resolve_db_path(input: &str) -> anyhow::Result<PathBuf> {
    let p = Path::new(input);
    if !p.is_absolute() {
        anyhow::bail!("路径必须是绝对路径: {}", input);
    }
    Ok(p.to_path_buf())
}

/// 解析 SQLite 文件:返回 SqlitePool + 文件大小
pub async fn connect(db_path: &Path) -> anyhow::Result<(SqlitePool, u64)> {
    let meta = tokio::fs::metadata(db_path).await?;
    if !meta.is_file() {
        anyhow::bail!("不是文件: {}", db_path.display());
    }
    let url = format!("sqlite://{}?mode=ro", db_path.display());
    let pool = tokio::time::timeout(CONNECT_TIMEOUT, SqlitePool::connect(&url))
        .await
        .map_err(|_| anyhow::anyhow!("连接超时"))?
        .map_err(|e| anyhow::anyhow!("连接失败: {}", e))?;
    Ok((pool, meta.len()))
}

/// 以可写模式连接,确保父目录存在
pub async fn connect_writable(db_path: &Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let url = format!("sqlite://{}", db_path.display());
    let pool = SqlitePool::connect(&url)
        .await
        .map_err(|e| anyhow::anyhow!("连接失败: {}", e))?;
    Ok(pool)
}

/// 列出所有用户表
pub async fn list_tables(db_path: &Path) -> anyhow::Result<Vec<TableInfo>> {
    let (target_pool, size) = connect(db_path).await?;
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&target_pool)
    .await?;
    let _ = target_pool.close().await;

    let mut out = Vec::with_capacity(rows.len());
    for (name,) in rows {
        // ponytail: 单库>100 表时改一次性 COUNT
        let (probe, _) = connect(db_path).await?;
        let count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM \"{}\"",
            name.replace('"', "\"\"")
        ))
        .fetch_one(&probe)
        .await
        .unwrap_or(0);
        let _ = probe.close().await;

        out.push(TableInfo {
            name,
            row_count: count,
            size_bytes: size as i64,
        });
    }
    Ok(out)
}

/// 拉表结构 + 分页数据
pub async fn table_data(
    db_path: &Path,
    table: &str,
    page: i64,
    page_size: i64,
) -> anyhow::Result<TableData> {
    if !is_safe_identifier(table) {
        anyhow::bail!("非法的表名: {}", table);
    }
    let pool = connect_writable(db_path).await?;
    // 拉列信息
    let pragma_rows = sqlx::query(&format!("PRAGMA table_info(\"{}\")", table.replace('"', "\"\"")))
        .fetch_all(&pool)
        .await?;
    let mut columns = Vec::new();
    let mut primary_key = Vec::new();
    for r in pragma_rows {
        let name: String = r.try_get("name")?;
        let ty: String = r.try_get("type").unwrap_or_default();
        let notnull: i64 = r.try_get("notnull").unwrap_or(0);
        let pk: i64 = r.try_get("pk").unwrap_or(0);
        let default_value: Option<String> = r.try_get("dflt_value").ok();
        if pk > 0 {
            primary_key.push(name.clone());
        }
        columns.push(ColumnInfo {
            name,
            r#type: ty,
            notnull: notnull != 0,
            pk: pk > 0,
            default_value,
        });
    }
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM \"{}\"",
        table.replace('"', "\"\"")
    ))
    .fetch_one(&pool)
    .await?;

    let offset = (page.max(1) - 1) * page_size.max(1);
    let limit = page_size.max(1).min(500);
    let data_rows = sqlx::query(&format!(
        "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
        table.replace('"', "\"\""),
        limit,
        offset
    ))
    .fetch_all(&pool)
    .await?;

    let mut json_rows = Vec::with_capacity(data_rows.len());
    for r in &data_rows {
        let mut map = serde_json::Map::new();
        for c in &columns {
            let v = row_value_to_json(r, c);
            map.insert(c.name.clone(), v);
        }
        json_rows.push(map);
    }
    let _ = pool.close().await;

    Ok(TableData {
        columns,
        primary_key,
        rows: json_rows,
        total,
        page: page.max(1),
        page_size: limit,
    })
}

/// 插入一行(values key 列名,value 已转 JSON)
pub async fn row_insert(
    db_path: &Path,
    table: &str,
    values: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<i64> {
    if !is_safe_identifier(table) {
        anyhow::bail!("非法的表名: {}", table);
    }
    if values.is_empty() {
        anyhow::bail!("values 不能为空");
    }
    let pool = connect_writable(db_path).await?;
    let mut cols = Vec::new();
    let mut placeholders = Vec::new();
    let mut binds: Vec<SqliteValue> = Vec::new();
    for (k, v) in values {
        if !is_safe_identifier(k) {
            anyhow::bail!("非法的列名: {}", k);
        }
        cols.push(format!("\"{}\"", k.replace('"', "\"\"")));
        placeholders.push("?");
        binds.push(SqliteValue::from_json(v));
    }
    let sql = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({})",
        table.replace('"', "\"\""),
        cols.join(","),
        placeholders.join(",")
    );
    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = bind_value(q, b);
    }
    let r = q.execute(&pool).await?;
    let last_id = r.last_insert_rowid();
    let _ = pool.close().await;
    Ok(last_id)
}

/// 更新一行(按主键定位)
pub async fn row_update(
    db_path: &Path,
    table: &str,
    pk: &serde_json::Map<String, serde_json::Value>,
    values: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<u64> {
    if !is_safe_identifier(table) {
        anyhow::bail!("非法的表名: {}", table);
    }
    if pk.is_empty() {
        anyhow::bail!("pk 不能为空");
    }
    if values.is_empty() {
        anyhow::bail!("values 不能为空");
    }
    let pool = connect_writable(db_path).await?;
    let mut sets = Vec::new();
    let mut binds: Vec<SqliteValue> = Vec::new();
    for (k, v) in values {
        if !is_safe_identifier(k) {
            anyhow::bail!("非法的列名: {}", k);
        }
        sets.push(format!("\"{}\"=?", k.replace('"', "\"\"")));
        binds.push(SqliteValue::from_json(v));
    }
    let mut wheres = Vec::new();
    for (k, v) in pk {
        if !is_safe_identifier(k) {
            anyhow::bail!("非法的列名: {}", k);
        }
        wheres.push(format!("\"{}\"=?", k.replace('"', "\"\"")));
        binds.push(SqliteValue::from_json(v));
    }
    let sql = format!(
        "UPDATE \"{}\" SET {} WHERE {}",
        table.replace('"', "\"\""),
        sets.join(","),
        wheres.join(" AND ")
    );
    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = bind_value(q, b);
    }
    let r = q.execute(&pool).await?;
    let n = r.rows_affected();
    let _ = pool.close().await;
    Ok(n)
}

/// 删除一行
pub async fn row_delete(
    db_path: &Path,
    table: &str,
    pk: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<u64> {
    if !is_safe_identifier(table) {
        anyhow::bail!("非法的表名: {}", table);
    }
    if pk.is_empty() {
        anyhow::bail!("pk 不能为空");
    }
    let pool = connect_writable(db_path).await?;
    let mut wheres = Vec::new();
    let mut binds: Vec<SqliteValue> = Vec::new();
    for (k, v) in pk {
        if !is_safe_identifier(k) {
            anyhow::bail!("非法的列名: {}", k);
        }
        wheres.push(format!("\"{}\"=?", k.replace('"', "\"\"")));
        binds.push(SqliteValue::from_json(v));
    }
    let sql = format!(
        "DELETE FROM \"{}\" WHERE {}",
        table.replace('"', "\"\""),
        wheres.join(" AND ")
    );
    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = bind_value(q, b);
    }
    let r = q.execute(&pool).await?;
    let n = r.rows_affected();
    let _ = pool.close().await;
    Ok(n)
}

/// 任意 SQL 查询(单语句,首词是 SELECT/PRAGMA/EXPLAIN)
pub async fn exec_sql(db_path: &Path, sql: &str) -> anyhow::Result<SqlResult> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        anyhow::bail!("SQL 不能为空");
    }
    // 禁止多语句(防止 ; 拼接)
    if trimmed.matches(';').count() > 1
        || (trimmed.matches(';').count() == 1 && !trimmed.ends_with(';'))
    {
        anyhow::bail!("仅支持单条 SQL 语句");
    }
    let head = trimmed.split_whitespace().next().unwrap_or("").to_uppercase();
    if !matches!(head.as_str(), "SELECT" | "PRAGMA" | "EXPLAIN" | "WITH") {
        anyhow::bail!("只允许 SELECT/PRAGMA/EXPLAIN/WITH,收到: {}", head);
    }
    let pool = connect_writable(db_path).await?;
    let rows_result = sqlx::query(trimmed.trim_end_matches(';')).fetch_all(&pool).await;
    let _ = pool.close().await;
    match rows_result {
        Ok(rows) => {
            let columns: Vec<String> = rows
                .first()
                .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
                .unwrap_or_default();
            let mut json_rows = Vec::with_capacity(rows.len());
            for r in &rows {
                let mut map = serde_json::Map::new();
                for (i, c) in r.columns().iter().enumerate() {
                    let v = decode_value(r, i, c);
                    map.insert(c.name().to_string(), v);
                }
                json_rows.push(map);
            }
            Ok(SqlResult {
                columns,
                rows: json_rows,
                rows_affected: 0,
                error: None,
            })
        }
        Err(e) => Ok(SqlResult {
            columns: vec![],
            rows: vec![],
            rows_affected: 0,
            error: Some(e.to_string()),
        }),
    }
}

/// 默认目录:exe 同级 server/sqlite/(预留,前端需要时再调用)
#[allow(dead_code)]
pub fn default_sqlite_dir() -> PathBuf {
    get_run_dir().join(DEFAULT_DIR)
}

// =============================================================
// helpers
// =============================================================

/// 仅允许 [A-Za-z0-9_]
fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 行任意列值 → JSON
fn row_value_to_json(
    r: &sqlx::sqlite::SqliteRow,
    col: &ColumnInfo,
) -> serde_json::Value {
    let ty = col.r#type.to_uppercase();
    // ponytail: TEXT 类声明列直接走 String 路径,避免 try_get::<Vec<u8>> fallback 误把 TEXT 解析成 BLOB
    let is_text = ty.contains("CHAR")
        || ty.contains("TEXT")
        || ty.contains("CLOB")
        || ty.contains("VARCHAR")
        || ty.contains("NVARCHAR")
        || ty.is_empty();
    if is_text {
        return r
            .try_get::<Option<String>, _>(col.name.as_str())
            .ok()
            .flatten()
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null);
    }
    // INTEGER 类
    if ty.contains("INT") {
        if let Ok(v) = r.try_get::<Option<i64>, _>(col.name.as_str()) {
            return v.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null);
        }
    }
    // REAL 类
    if ty.contains("REAL") || ty.contains("FLOAT") || ty.contains("DOUB") {
        if let Ok(v) = r.try_get::<Option<f64>, _>(col.name.as_str()) {
            return v.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null);
        }
    }
    // 显式 BLOB 类(或无法识别的类型)
    if let Ok(v) = r.try_get::<Option<Vec<u8>>, _>(col.name.as_str()) {
        if let Some(b) = v {
            return serde_json::Value::String(format!("<BLOB {} bytes>", b.len()));
        }
        return serde_json::Value::Null;
    }
    // 兜底
    r.try_get::<Option<String>, _>(col.name.as_str())
        .ok()
        .flatten()
        .map(serde_json::Value::from)
        .unwrap_or(serde_json::Value::Null)
}

fn decode_value(
    r: &sqlx::sqlite::SqliteRow,
    idx: usize,
    col: &sqlx::sqlite::SqliteColumn,
) -> serde_json::Value {
    let name = col.name();
    if let Ok(v) = r.try_get::<Option<i64>, _>(name) {
        return v.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(v) = r.try_get::<Option<f64>, _>(name) {
        return v.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null);
    }
    if let Ok(v) = r.try_get::<Option<bool>, _>(name) {
        return v.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null);
    }
    let _ = idx;
    r.try_get::<Option<String>, _>(name)
        .ok()
        .flatten()
        .map(serde_json::Value::from)
        .unwrap_or(serde_json::Value::Null)
}

#[derive(Debug, Clone)]
enum SqliteValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Bool(bool),
}

impl SqliteValue {
    fn from_json(v: &serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => SqliteValue::Null,
            serde_json::Value::Bool(b) => SqliteValue::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    SqliteValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    SqliteValue::Real(f)
                } else {
                    SqliteValue::Text(n.to_string())
                }
            }
            serde_json::Value::String(s) => SqliteValue::Text(s.clone()),
            other => SqliteValue::Text(other.to_string()),
        }
    }
}

fn bind_value<'q>(
    q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    v: &'q SqliteValue,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match v {
        SqliteValue::Null => q.bind(Option::<i64>::None),
        SqliteValue::Integer(i) => q.bind(*i),
        SqliteValue::Real(f) => q.bind(*f),
        // ponytail: sqlx 0.7 用 &str bind 会被推断为 BLOB,改用 String 才是 TEXT
        SqliteValue::Text(s) => q.bind(s.clone()),
        SqliteValue::Bool(b) => q.bind(*b),
    }
}

// =============================================================
// DDL:表与字段
// =============================================================

/// 创建新表:接受完整 CREATE TABLE SQL 字符串
pub async fn create_table(db_path: &Path, sql: &str) -> anyhow::Result<()> {
    let upper = sql.trim_start().to_uppercase();
    if !upper.starts_with("CREATE TABLE") {
        anyhow::bail!("仅接受 CREATE TABLE 语句");
    }
    if sql.matches(';').count() > 1 {
        anyhow::bail!("仅支持单条语句");
    }
    let pool = connect_writable(db_path).await?;
    sqlx::query(sql.trim_end_matches(';')).execute(&pool).await?;
    let _ = pool.close().await;
    Ok(())
}

/// 重命名表
pub async fn rename_table(db_path: &Path, old: &str, new: &str) -> anyhow::Result<()> {
    if !is_safe_identifier(old) || !is_safe_identifier(new) {
        anyhow::bail!("非法的表名");
    }
    let pool = connect_writable(db_path).await?;
    let sql = format!(
        "ALTER TABLE \"{}\" RENAME TO \"{}\"",
        old.replace('"', "\"\""),
        new.replace('"', "\"\"")
    );
    sqlx::query(&sql).execute(&pool).await?;
    let _ = pool.close().await;
    Ok(())
}

/// 删除表
pub async fn drop_table(db_path: &Path, name: &str) -> anyhow::Result<()> {
    if !is_safe_identifier(name) {
        anyhow::bail!("非法的表名");
    }
    let pool = connect_writable(db_path).await?;
    let sql = format!("DROP TABLE \"{}\"", name.replace('"', "\"\""));
    sqlx::query(&sql).execute(&pool).await?;
    let _ = pool.close().await;
    Ok(())
}

/// 添加列(col_def 形如 "name TEXT NOT NULL DEFAULT 0",只校验列名)
pub async fn add_column(db_path: &Path, table: &str, col_def: &str) -> anyhow::Result<()> {
    if !is_safe_identifier(table) {
        anyhow::bail!("非法的表名");
    }
    let col_name = col_def.split_whitespace().next().unwrap_or("");
    if !is_safe_identifier(col_name) {
        anyhow::bail!("非法的列名");
    }
    // 简单禁止危险关键字
    let upper = col_def.to_uppercase();
    for bad in [";", "--", "/*", "*/", "DROP ", "DELETE ", "UPDATE ", "ALTER "] {
        if upper.contains(bad) {
            anyhow::bail!("列定义包含非法字符: {}", bad);
        }
    }
    let pool = connect_writable(db_path).await?;
    let sql = format!(
        "ALTER TABLE \"{}\" ADD COLUMN {}",
        table.replace('"', "\"\""),
        col_def
    );
    sqlx::query(&sql).execute(&pool).await?;
    let _ = pool.close().await;
    Ok(())
}

/// 重命名列(SQLite 3.25+)
pub async fn rename_column(
    db_path: &Path,
    table: &str,
    old: &str,
    new: &str,
) -> anyhow::Result<()> {
    if !is_safe_identifier(table) || !is_safe_identifier(old) || !is_safe_identifier(new) {
        anyhow::bail!("非法的标识符");
    }
    let pool = connect_writable(db_path).await?;
    let sql = format!(
        "ALTER TABLE \"{}\" RENAME COLUMN \"{}\" TO \"{}\"",
        table.replace('"', "\"\""),
        old.replace('"', "\"\""),
        new.replace('"', "\"\"")
    );
    sqlx::query(&sql).execute(&pool).await?;
    let _ = pool.close().await;
    Ok(())
}

/// 删除列(SQLite 3.35+)
pub async fn drop_column(db_path: &Path, table: &str, col: &str) -> anyhow::Result<()> {
    if !is_safe_identifier(table) || !is_safe_identifier(col) {
        anyhow::bail!("非法的标识符");
    }
    let pool = connect_writable(db_path).await?;
    let sql = format!(
        "ALTER TABLE \"{}\" DROP COLUMN \"{}\"",
        table.replace('"', "\"\""),
        col.replace('"', "\"\"")
    );
    sqlx::query(&sql).execute(&pool).await?;
    let _ = pool.close().await;
    Ok(())
}
