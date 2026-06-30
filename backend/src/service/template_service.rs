use crate::dto::{CreateTemplateRequest, UpdateTemplateRequest};
use crate::model::Template;
use crate::AppState;

/// 获取所有配置模板
pub async fn get_all_templates(state: &AppState) -> anyhow::Result<Vec<Template>> {
    let templates = sqlx::query_as::<_, Template>(
        "SELECT * FROM templates ORDER BY created_at DESC"
    )
    .fetch_all(state.db.pool())
    .await?;
    Ok(templates)
}

/// 获取单个配置模板
pub async fn get_template(state: &AppState, id: i64) -> anyhow::Result<Option<Template>> {
    let template = sqlx::query_as::<_, Template>(
        "SELECT * FROM templates WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await?;
    Ok(template)
}

/// 创建配置模板
pub async fn create_template(
    state: &AppState,
    req: CreateTemplateRequest,
) -> anyhow::Result<Template> {
    let template = sqlx::query_as::<_, Template>(
        r#"
        INSERT INTO templates (name, description, config, variables)
        VALUES (?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.config)
    .bind(&req.variables)
    .fetch_one(state.db.pool())
    .await?;

    Ok(template)
}

/// 更新配置模板
pub async fn update_template(
    state: &AppState,
    id: i64,
    req: UpdateTemplateRequest,
) -> anyhow::Result<Option<Template>> {
    let existing = get_template(state, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let name = req.name.unwrap_or(existing.name);
    let description = req.description.or(existing.description);
    let config = req.config.unwrap_or(existing.config);
    let variables = req.variables.or(existing.variables);

    let template = sqlx::query_as::<_, Template>(
        r#"
        UPDATE templates
        SET name = ?, description = ?, config = ?, variables = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&config)
    .bind(&variables)
    .bind(id)
    .fetch_optional(state.db.pool())
    .await?;

    Ok(template)
}

/// 删除配置模板
pub async fn delete_template(state: &AppState, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM templates WHERE id = ?")
        .bind(id)
        .execute(state.db.pool())
        .await?;

    Ok(result.rows_affected() > 0)
}

/// 应用模板变量
pub fn apply_template_variables(
    config: &str,
    variables: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = config.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}
