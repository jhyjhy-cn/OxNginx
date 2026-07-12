use sqlx::SqlitePool;

use crate::modules::site::entity::site::Site;

pub async fn list_all_sites(pool: &SqlitePool) -> sqlx::Result<Vec<Site>> {
    sqlx::query_as::<_, Site>("SELECT * FROM site_sites ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn find_site_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Site>> {
    sqlx::query_as::<_, Site>("SELECT * FROM site_sites WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_site_returning(
    pool: &SqlitePool,
    req: &crate::modules::common::dto::CreateSiteRequest,
    ssl_value: i32,
) -> sqlx::Result<Site> {
    sqlx::query_as::<_, Site>(
        r#"
        INSERT INTO site_sites (name, server_name, listen, ssl, certificate_path, key_path, proxy_pass, root_path, remark, expire_time, rewrite_rules, redirect_rules, hotlink_config, log_access_path, log_error_path)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&req.name)
    .bind(&req.server_name)
    .bind(&req.listen)
    .bind(ssl_value)
    .bind(&req.certificate_path)
    .bind(&req.key_path)
    .bind(&req.proxy_pass)
    .bind(&req.root_path)
    .bind(&req.remark)
    .bind(&req.expire_time)
    .bind(&req.rewrite_rules)
    .bind(&req.redirect_rules)
    .bind(&req.hotlink_config)
    .bind(&req.log_access_path)
    .bind(&req.log_error_path)
    .fetch_one(pool)
    .await
}

pub async fn update_site_returning(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    server_name: &str,
    listen: &str,
    ssl: i32,
    certificate_path: Option<&String>,
    key_path: Option<&String>,
    proxy_pass: Option<&String>,
    root_path: Option<&String>,
    remark: Option<&String>,
    expire_time: Option<&String>,
    rewrite_rules: Option<&String>,
    redirect_rules: Option<&String>,
    hotlink_config: Option<&String>,
    log_access_path: Option<&String>,
    log_error_path: Option<&String>,
    status: i32,
) -> sqlx::Result<Option<Site>> {
    sqlx::query_as::<_, Site>(
        r#"
        UPDATE site_sites
        SET name = ?, server_name = ?, listen = ?, ssl = ?, certificate_path = ?, key_path = ?, proxy_pass = ?, root_path = ?, remark = ?, expire_time = ?, rewrite_rules = ?, redirect_rules = ?, hotlink_config = ?, log_access_path = ?, log_error_path = ?, status = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(server_name)
    .bind(listen)
    .bind(ssl)
    .bind(certificate_path)
    .bind(key_path)
    .bind(proxy_pass)
    .bind(root_path)
    .bind(remark)
    .bind(expire_time)
    .bind(rewrite_rules)
    .bind(redirect_rules)
    .bind(hotlink_config)
    .bind(log_access_path)
    .bind(log_error_path)
    .bind(status)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_site(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query("DELETE FROM site_sites WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}