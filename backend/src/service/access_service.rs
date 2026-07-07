use crate::dto::{CreateAccessRuleRequest, UpdateAccessRuleRequest};
use crate::model::AccessRule;
use crate::AppState;

/// 获取所有访问控制规则
pub async fn get_all_rules(state: &AppState) -> anyhow::Result<Vec<AccessRule>> {
    let rules = sqlx::query_as::<_, AccessRule>(
        "SELECT * FROM sys_access_rules ORDER BY created_at DESC"
    )
    .fetch_all(state.db.pool())
    .await?;
    Ok(rules)
}

/// 获取单个访问控制规则
pub async fn get_rule(state: &AppState, id: i64) -> anyhow::Result<Option<AccessRule>> {
    let rule = sqlx::query_as::<_, AccessRule>(
        "SELECT * FROM sys_access_rules WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await?;
    Ok(rule)
}

/// 创建访问控制规则
pub async fn create_rule(
    state: &AppState,
    req: CreateAccessRuleRequest,
) -> anyhow::Result<AccessRule> {
    let rule = sqlx::query_as::<_, AccessRule>(
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
    .fetch_one(state.db.pool())
    .await?;

    Ok(rule)
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

    let rule = sqlx::query_as::<_, AccessRule>(
        r#"
        UPDATE sys_access_rules
        SET site_id = ?, rule_type = ?, value = ?, description = ?, status = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(site_id)
    .bind(&rule_type)
    .bind(&value)
    .bind(&description)
    .bind(&status)
    .bind(id)
    .fetch_optional(state.db.pool())
    .await?;

    Ok(rule)
}

/// 删除访问控制规则
pub async fn delete_rule(state: &AppState, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM sys_access_rules WHERE id = ?")
        .bind(id)
        .execute(state.db.pool())
        .await?;

    Ok(result.rows_affected() > 0)
}
