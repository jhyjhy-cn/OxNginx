use sqlx::SqlitePool;

use crate::modules::site::entity::access::AccessRule;

pub async fn list_all_rules(pool: &SqlitePool) -> sqlx::Result<Vec<AccessRule>> {
    sqlx::query_as::<_, AccessRule>("SELECT * FROM sys_access_rules ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn find_rule_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<AccessRule>> {
    sqlx::query_as::<_, AccessRule>("SELECT * FROM sys_access_rules WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_rule_returning(
    pool: &SqlitePool,
    req: &crate::modules::common::dto::CreateAccessRuleRequest,
) -> sqlx::Result<AccessRule> {
    sqlx::query_as::<_, AccessRule>(
        r#"
        INSERT INTO sys_access_rules (site_id, rule_type, value, description)
        VALUES (?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(req.site_id)
    .bind(&req.rule_type)
    .bind(&req.value)
    .bind(&req.description)
    .fetch_one(pool)
    .await
}

pub async fn update_rule_returning(
    pool: &SqlitePool,
    id: i64,
    site_id: Option<i64>,
    rule_type: &str,
    value: &str,
    description: Option<&String>,
    status: &str,
) -> sqlx::Result<Option<AccessRule>> {
    sqlx::query_as::<_, AccessRule>(
        r#"
        UPDATE sys_access_rules
        SET site_id = ?, rule_type = ?, value = ?, description = ?, status = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(site_id)
    .bind(rule_type)
    .bind(value)
    .bind(description)
    .bind(status)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_rule(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_access_rules WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}