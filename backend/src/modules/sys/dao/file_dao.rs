use sqlx::SqlitePool;

use crate::modules::sys::entity::file::File;

pub async fn page_files(
    pool: &SqlitePool,
    keyword: Option<&str>,
    suffix: Option<&str>,
    provider: Option<&str>,
    page: i64,
    page_size: i64,
) -> sqlx::Result<(Vec<File>, i64)> {
    let offset = (page - 1) * page_size;

    let like_kw = keyword.map(|k| format!("%{}%", k));

    let rows = sqlx::query_as::<_, File>(
        "SELECT id, name, original_name, suffix, size, mime_type, md5, path, provider, dept_id, created_at, updated_at, created_by, updated_by
         FROM sys_files
         WHERE (? IS NULL OR original_name LIKE ? OR name LIKE ?)
           AND (? IS NULL OR suffix = ?)
           AND (? IS NULL OR provider = ?)
         ORDER BY id DESC
         LIMIT ? OFFSET ?",
    )
    .bind(like_kw.as_ref())
    .bind(like_kw.as_ref())
    .bind(like_kw.as_ref())
    .bind(suffix)
    .bind(suffix)
    .bind(provider)
    .bind(provider)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sys_files
         WHERE (? IS NULL OR original_name LIKE ? OR name LIKE ?)
           AND (? IS NULL OR suffix = ?)
           AND (? IS NULL OR provider = ?)",
    )
    .bind(like_kw.as_ref())
    .bind(like_kw.as_ref())
    .bind(like_kw.as_ref())
    .bind(suffix)
    .bind(suffix)
    .bind(provider)
    .bind(provider)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn find_file_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<File>> {
    sqlx::query_as::<_, File>(
        "SELECT id, name, original_name, suffix, size, mime_type, md5, path, provider, dept_id, created_at, updated_at, created_by, updated_by
         FROM sys_files WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_file_returning_id(
    pool: &SqlitePool,
    name: &str,
    original_name: &str,
    suffix: &str,
    size: i64,
    mime_type: Option<&str>,
    md5: Option<&str>,
    path: &str,
    provider: &str,
    dept_id: Option<i64>,
    created_by: Option<i64>,
) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_files (name, original_name, suffix, size, mime_type, md5, path, provider, dept_id, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(original_name)
    .bind(suffix)
    .bind(size)
    .bind(mime_type)
    .bind(md5)
    .bind(path)
    .bind(provider)
    .bind(dept_id)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

pub async fn delete_file(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_files WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}

pub async fn delete_files(pool: &SqlitePool, ids: &[i64]) -> sqlx::Result<Vec<crate::modules::sys::entity::file::File>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    // 先查记录以便删文件
    let placeholders = std::iter::repeat("?").take(ids.len()).collect::<Vec<_>>().join(",");
    let sql_select = format!("SELECT id, name, original_name, suffix, size, mime_type, md5, path, provider, dept_id, remark, version, created_at, updated_at, created_by, updated_by FROM sys_files WHERE id IN ({})", placeholders);
    let mut q = sqlx::query_as::<_, crate::modules::sys::entity::file::File>(&sql_select);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(pool).await?;

    let sql_del = format!("DELETE FROM sys_files WHERE id IN ({})", placeholders);
    let mut qd = sqlx::query(&sql_del);
    for id in ids {
        qd = qd.bind(id);
    }
    qd.execute(pool).await?;

    Ok(rows)
}