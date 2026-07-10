use sqlx::SqlitePool;

use crate::modules::sys::entity::menu::Menu;

pub async fn list_menus(pool: &SqlitePool) -> sqlx::Result<Vec<Menu>> {
    sqlx::query_as::<_, Menu>("SELECT * FROM sys_menus ORDER BY sort, id")
        .fetch_all(pool)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_menu_returning_id(
    pool: &SqlitePool,
    name: &str,
    title: &str,
    parent_id: Option<i64>,
    icon: Option<&str>,
    path: Option<&str>,
    component: Option<&str>,
    menu_type: i32,
    permission: Option<&str>,
    sort: i32,
) -> sqlx::Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO sys_menus (name, title, parent_id, icon, path, component, type, permission, sort)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(title)
    .bind(parent_id)
    .bind(icon)
    .bind(path)
    .bind(component)
    .bind(menu_type)
    .bind(permission)
    .bind(sort)
    .fetch_one(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn update_menu_fields(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    title: Option<&str>,
    parent_id: Option<Option<i64>>,
    icon: Option<Option<&str>>,
    path: Option<Option<&str>>,
    component: Option<Option<&str>>,
    permission: Option<Option<&str>>,
    sort: Option<i32>,
) -> sqlx::Result<()> {
    if let Some(n) = name {
        sqlx::query("UPDATE sys_menus SET name=? WHERE id=?")
            .bind(n).bind(id).execute(pool).await?;
    }
    if let Some(t) = title {
        sqlx::query("UPDATE sys_menus SET title=? WHERE id=?")
            .bind(t).bind(id).execute(pool).await?;
    }
    if let Some(p) = parent_id {
        sqlx::query("UPDATE sys_menus SET parent_id=? WHERE id=?")
            .bind(p).bind(id).execute(pool).await?;
    }
    if let Some(i) = icon {
        sqlx::query("UPDATE sys_menus SET icon=? WHERE id=?")
            .bind(i).bind(id).execute(pool).await?;
    }
    if let Some(p) = path {
        sqlx::query("UPDATE sys_menus SET path=? WHERE id=?")
            .bind(p).bind(id).execute(pool).await?;
    }
    if let Some(c) = component {
        sqlx::query("UPDATE sys_menus SET component=? WHERE id=?")
            .bind(c).bind(id).execute(pool).await?;
    }
    if let Some(pm) = permission {
        sqlx::query("UPDATE sys_menus SET permission=? WHERE id=?")
            .bind(pm).bind(id).execute(pool).await?;
    }
    if let Some(s) = sort {
        sqlx::query("UPDATE sys_menus SET sort=? WHERE id=?")
            .bind(s).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn delete_menus(pool: &SqlitePool, ids: &[i64]) -> sqlx::Result<usize> {
    let mut all_ids: Vec<i64> = ids.to_vec();
    let mut frontier = ids.to_vec();

    while !frontier.is_empty() {
        let placeholders = vec!["?"; frontier.len()].join(",");
        let sql = format!("SELECT id FROM sys_menus WHERE parent_id IN ({})", placeholders);
        let mut q = sqlx::query_as::<_, (i64,)>(&sql);
        for id in &frontier {
            q = q.bind(id);
        }
        let children: Vec<(i64,)> = q.fetch_all(pool).await?;
        let child_ids: Vec<i64> = children.into_iter().map(|(c,)| c).collect();
        for c in &child_ids {
            if !all_ids.contains(c) {
                all_ids.push(*c);
            }
        }
        frontier = child_ids;
    }

    if all_ids.is_empty() {
        return Ok(0);
    }

    let placeholders = vec!["?"; all_ids.len()].join(",");
    let sql = format!("DELETE FROM sys_role_menus WHERE menu_id IN ({})", placeholders);
    let mut q = sqlx::query(&sql);
    for id in &all_ids {
        q = q.bind(id);
    }
    q.execute(pool).await?;

    let sql = format!("DELETE FROM sys_menus WHERE id IN ({})", placeholders);
    let mut q = sqlx::query(&sql);
    for id in &all_ids {
        q = q.bind(id);
    }
    let n = q.execute(pool).await?.rows_affected();
    Ok(n as usize)
}