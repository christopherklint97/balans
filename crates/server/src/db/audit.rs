use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Log an action to the audit trail.
pub async fn log_action(
    pool: &SqlitePool,
    entity_type: &str,
    entity_id: &str,
    action: &str,
    details: Option<&str>,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO audit_log (id, entity_type, entity_id, action, details, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(details)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(())
}

/// Log an action within a transaction.
pub async fn log_action_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entity_type: &str,
    entity_id: &str,
    action: &str,
    details: Option<&str>,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO audit_log (id, entity_type, entity_id, action, details, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(details)
    .bind(&now)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
