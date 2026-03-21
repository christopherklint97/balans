use sqlx::SqlitePool;

use crate::error::AppError;

/// Role hierarchy for company access: owner > admin > member > viewer
fn role_level(role: &str) -> i32 {
    match role {
        "owner" => 4,
        "admin" => 3,
        "member" => 2,
        "viewer" => 1,
        _ => 0,
    }
}

/// Verify that a user has at least `min_role` access to a company.
pub async fn verify_company_access(
    pool: &SqlitePool,
    user_id: &str,
    company_id: &str,
    min_role: &str,
) -> Result<String, AppError> {
    // System admins always have access
    let is_admin = sqlx::query_scalar::<_, String>(
        "SELECT role FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if is_admin.as_deref() == Some("admin") {
        return Ok("admin".to_string());
    }

    let row = sqlx::query_scalar::<_, String>(
        "SELECT role FROM user_companies WHERE user_id = ? AND company_id = ?",
    )
    .bind(user_id)
    .bind(company_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(role) if role_level(&role) >= role_level(min_role) => Ok(role),
        Some(_) => Err(AppError::Forbidden(
            "Otillräcklig behörighet för detta företag".into(),
        )),
        None => Err(AppError::Forbidden(
            "Ingen åtkomst till detta företag".into(),
        )),
    }
}

/// Look up the company_id for a fiscal year, then verify access.
pub async fn verify_fiscal_year_access(
    pool: &SqlitePool,
    user_id: &str,
    fiscal_year_id: &str,
    min_role: &str,
) -> Result<String, AppError> {
    let company_id = sqlx::query_scalar::<_, String>(
        "SELECT company_id FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Räkenskapsåret hittades inte".into()))?;

    verify_company_access(pool, user_id, &company_id, min_role).await
}
