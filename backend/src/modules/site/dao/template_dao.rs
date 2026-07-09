use sqlx::SqlitePool;

use crate::modules::site::entity::template::Template;

pub async fn list_all_templates(pool: &SqlitePool) -> sqlx::Result<Vec<Template>> {
    sqlx::query_as::<_, Template>("SELECT * FROM sys_templates ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn find_template_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Template>> {
    sqlx::query_as::<_, Template>("SELECT * FROM sys_templates WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_template_returning(
    pool: &SqlitePool,
    req: &crate::modules::common::dto::CreateTemplateRequest,
) -> sqlx::Result<Template> {
    sqlx::query_as::<_, Template>(
        r#"
        INSERT INTO sys_templates (name, description, config, variables)
        VALUES (?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.config)
    .bind(&req.variables)
    .fetch_one(pool)
    .await
}

pub async fn update_template_returning(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    description: Option<&String>,
    config: &str,
    variables: Option<&String>,
) -> sqlx::Result<Option<Template>> {
    sqlx::query_as::<_, Template>(
        r#"
        UPDATE sys_templates
        SET name = ?, description = ?, config = ?, variables = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(config)
    .bind(variables)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_template(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM sys_templates WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}