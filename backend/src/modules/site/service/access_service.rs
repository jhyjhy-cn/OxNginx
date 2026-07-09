use crate::modules::common::dto::{CreateAccessRuleRequest, UpdateAccessRuleRequest};
use crate::modules::site::dao::access_dao;
use crate::modules::site::entity::access::AccessRule;
use crate::AppState;

/// 获取所有访问控制规则
pub async fn get_all_rules(state: &AppState) -> anyhow::Result<Vec<AccessRule>> {
    Ok(access_dao::list_all_rules(state.db.pool()).await?)
}

/// 获取单个访问控制规则
pub async fn get_rule(state: &AppState, id: i64) -> anyhow::Result<Option<AccessRule>> {
    Ok(access_dao::find_rule_by_id(state.db.pool(), id).await?)
}

/// 创建访问控制规则
pub async fn create_rule(
    state: &AppState,
    req: CreateAccessRuleRequest,
) -> anyhow::Result<AccessRule> {
    Ok(access_dao::insert_rule_returning(state.db.pool(), &req).await?)
}

/// 更新访问控制规则
pub async fn update_rule(
    state: &AppState,
    id: i64,
    req: UpdateAccessRuleRequest,
) -> anyhow::Result<Option<AccessRule>> {
    let existing = get_rule(state, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let site_id = req.site_id.or(existing.site_id);
    let rule_type = req.rule_type.unwrap_or(existing.rule_type);
    let value = req.value.unwrap_or(existing.value);
    let description = req.description.or(existing.description);
    let status = req.status.unwrap_or(existing.status);

    Ok(access_dao::update_rule_returning(
        state.db.pool(),
        id,
        site_id,
        &rule_type,
        &value,
        description.as_ref(),
        &status,
    )
    .await?)
}

/// 删除访问控制规则
pub async fn delete_rule(state: &AppState, id: i64) -> anyhow::Result<bool> {
    Ok(access_dao::delete_rule(state.db.pool(), id).await? > 0)
}