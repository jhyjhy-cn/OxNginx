use crate::modules::common::dto::{CreateTemplateRequest, UpdateTemplateRequest};
use crate::modules::site::dao::template_dao;
use crate::modules::site::entity::template::Template;
use crate::AppState;

/// 获取所有配置模板
pub async fn get_all_templates(state: &AppState) -> anyhow::Result<Vec<Template>> {
    Ok(template_dao::list_all_templates(state.db.pool()).await?)
}

/// 获取单个配置模板
pub async fn get_template(state: &AppState, id: i64) -> anyhow::Result<Option<Template>> {
    Ok(template_dao::find_template_by_id(state.db.pool(), id).await?)
}

/// 创建配置模板
pub async fn create_template(
    state: &AppState,
    req: CreateTemplateRequest,
) -> anyhow::Result<Template> {
    Ok(template_dao::insert_template_returning(state.db.pool(), &req).await?)
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

    Ok(template_dao::update_template_returning(
        state.db.pool(),
        id,
        &name,
        description.as_ref(),
        &config,
        variables.as_ref(),
    )
    .await?)
}

/// 删除配置模板
pub async fn delete_template(state: &AppState, id: i64) -> anyhow::Result<bool> {
    Ok(template_dao::delete_template(state.db.pool(), id).await? > 0)
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