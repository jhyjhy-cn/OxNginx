use anyhow::Result;
use sqlx::SqlitePool;

use crate::model::Role;

// ============== 角色 CRUD ==============

/// 分页查询角色
pub async fn list_roles_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> Result<(Vec<Role>, i64)> {
    let offset = (page - 1).max(0) * page_size;
    let like = keyword.map(|k| format!("%{}%", k));
    let (total, rows) = if let Some(ref pattern) = like {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sys_roles WHERE name LIKE ? OR code LIKE ?",
        )
        .bind(pattern)
        .bind(pattern)
        .fetch_one(pool)
        .await?;
        let rows = sqlx::query_as::<_, Role>(
            "SELECT * FROM sys_roles WHERE name LIKE ? OR code LIKE ? ORDER BY id LIMIT ? OFFSET ?",
        )
        .bind(pattern)
        .bind(pattern)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    } else {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sys_roles")
                .fetch_one(pool)
                .await?;
        let rows = sqlx::query_as::<_, Role>(
            "SELECT * FROM sys_roles ORDER BY id LIMIT ? OFFSET ?",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        (total, rows)
    };
    Ok((rows, total))
}

/// 全量角色列表（给下拉框用）
pub async fn list_roles(pool: &SqlitePool) -> Result<Vec<Role>> {
    Ok(sqlx::query_as::<_, Role>(
        "SELECT * FROM sys_roles WHERE status='enabled' ORDER BY id",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_role(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    remark: Option<&str>,
) -> Result<i64> {
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_roles (code, name, remark) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(code)
    .bind(name)
    .bind(remark)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update_role(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    remark: Option<&str>,
    status: Option<&str>,
) -> Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_roles SET name=? WHERE id=?")
            .bind(n).bind(id).execute(pool).await?;
    }
    if let Some(r) = remark {
        sqlx::query("UPDATE sys_roles SET remark=? WHERE id=?")
            .bind(r).bind(id).execute(pool).await?;
    }
    if let Some(s) = status {
        sqlx::query("UPDATE sys_roles SET status=? WHERE id=?")
            .bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_role(pool: &SqlitePool, id: i64) -> Result<bool> {
    let n = sqlx::query("DELETE FROM sys_roles WHERE id=? AND code != 'super_admin'")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n > 0)
}

pub async fn set_role_menus(pool: &SqlitePool, role_id: i64, menu_ids: &[i64]) -> Result<()> {
    sqlx::query("DELETE FROM sys_role_menus WHERE role_id=?")
        .bind(role_id)
        .execute(pool)
        .await?;
    for mid in menu_ids {
        sqlx::query("INSERT OR IGNORE INTO sys_role_menus (role_id, menu_id) VALUES (?, ?)")
            .bind(role_id)
            .bind(mid)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn get_role_menus(pool: &SqlitePool, role_id: i64) -> Result<Vec<i64>> {
    let rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT menu_id FROM sys_role_menus WHERE role_id=? ORDER BY menu_id",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(m,)| m).collect())
}
