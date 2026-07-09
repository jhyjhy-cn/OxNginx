use sqlx::SqlitePool;

use crate::modules::site::entity::certificate::Certificate;

pub async fn list_all_certs(pool: &SqlitePool) -> sqlx::Result<Vec<Certificate>> {
    sqlx::query_as::<_, Certificate>("SELECT * FROM sys_certificates ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn find_cert_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Certificate>> {
    sqlx::query_as::<_, Certificate>("SELECT * FROM sys_certificates WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_cert_by_domain(pool: &SqlitePool, domain: &str) -> sqlx::Result<Option<Certificate>> {
    sqlx::query_as::<_, Certificate>("SELECT * FROM sys_certificates WHERE domain = ?")
        .bind(domain)
        .fetch_optional(pool)
        .await
}

pub async fn update_cert_paths(
    pool: &SqlitePool,
    id: i64,
    cert_path: &str,
    key_path: &str,
) -> sqlx::Result<()> {
    sqlx::query("UPDATE sys_certificates SET cert_path = ?, key_path = ? WHERE id = ?")
        .bind(cert_path)
        .bind(key_path)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_cert_returning(
    pool: &SqlitePool,
    domain: &str,
    cert_path: &str,
    key_path: &str,
) -> sqlx::Result<Certificate> {
    sqlx::query_as::<_, Certificate>(
        r#"
        INSERT INTO sys_certificates (domain, issuer, cert_path, key_path, auto_renew)
        VALUES (?, 'Let''s Encrypt', ?, ?, 1)
        RETURNING *
        "#,
    )
    .bind(domain)
    .bind(cert_path)
    .bind(key_path)
    .fetch_one(pool)
    .await
}