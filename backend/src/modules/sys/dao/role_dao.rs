use sqlx::SqlitePool;

use crate::modules::sys::entity::role::Role;

/// 分页查询：返回 (rows, total)
pub async fn list_roles_paged(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
    keyword: Option<&str>,
) -> sqlx::Result<(Vec<Role>, i64)> {
    let offset = (page - 1).max(0) * page_size;
    let like = keyword.map(|k| format!("%{}%", k));
    if let Some(ref pattern) = like {
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
        Ok((rows, total))
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_roles")
            .fetch_one(pool)
            .await?;
        let rows = sqlx::query_as::<_, Role>(
            "SELECT * FROM sys_roles ORDER BY id LIMIT ? OFFSET ?",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok((rows, total))
    }
}


pub async fn insert_role_returning_id(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    remark: Option<&str>,
) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_roles (code, name, remark) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(code)
    .bind(name)
    .bind(remark)
    .fetch_one(pool)
    .await
}

pub async fn update_role_fields(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    remark: Option<&str>,
    status: Option<i32>,
) -> sqlx::Result<()> {
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

pub async fn delete_role_protect_super_admin(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_roles WHERE id=? AND code != 'super_admin'")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}

/// 批量删角色: 过滤掉 super_admin,返回实际删除条数
pub async fn delete_roles_protect_super_admin(pool: &SqlitePool, ids: &[i64]) -> sqlx::Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    // ponytail: 单 SQL IN (...) 走 N 个占位符,避免拼接
    let placeholders = std::iter::repeat("?").take(ids.len()).collect::<Vec<_>>().join(",");
    let sql = format!(
        "DELETE FROM sys_roles WHERE code != 'super_admin' AND id IN ({})",
        placeholders
    );
    let mut q = sqlx::query(&sql);
    for id in ids {
        q = q.bind(id);
    }
    let r = q.execute(pool).await?;
    Ok(r.rows_affected())
}

pub async fn replace_role_menus(
    pool: &SqlitePool,
    role_id: i64,
    menu_ids: &[i64],
) -> sqlx::Result<()> {
    tracing::debug!(role_id, menu_ids_len = menu_ids.len(), "replace_role_menus");
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM sys_role_menus WHERE role_id=?")
        .bind(role_id)
        .execute(&mut *tx)
        .await?;

    if !menu_ids.is_empty() {
        let placeholders = std::iter::repeat("(?, ?)")
            .take(menu_ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT INTO sys_role_menus (role_id, menu_id) VALUES {placeholders}"
        );
        let mut q = sqlx::query(&sql);
        for mid in menu_ids {
            q = q.bind(role_id).bind(*mid);
        }
        q.execute(&mut *tx).await?;
    }

    tx.commit().await
}

pub async fn list_role_menu_ids(pool: &SqlitePool, role_id: i64) -> sqlx::Result<Vec<i64>> {
    let rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT menu_id FROM sys_role_menus WHERE role_id=? ORDER BY menu_id",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(m,)| m).collect())
}