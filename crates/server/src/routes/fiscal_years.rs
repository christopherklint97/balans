use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Datelike;
use crate::access::{verify_company_access, verify_fiscal_year_access};
use crate::auth::middleware::AuthUser;
use crate::config::AppState;

use crate::error::AppError;
use crate::models::fiscal_year::{CreateFiscalYear, FiscalYear};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/companies/{company_id}/fiscal-years",
            post(create_fiscal_year).get(list_fiscal_years),
        )
        .route("/fiscal-years/{id}", get(get_fiscal_year))
}

async fn create_fiscal_year(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(company_id): Path<String>,
    Json(input): Json<CreateFiscalYear>,
) -> Result<Json<FiscalYear>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "member").await?;

    // Verify company exists
    let exists = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM companies WHERE id = ?")
        .bind(&company_id)
        .fetch_one(&state.pool)
        .await?;
    if exists == 0 {
        return Err(AppError::NotFound(format!(
            "Company {company_id} not found"
        )));
    }

    // Validate dates
    let start = chrono::NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Invalid start_date format, use YYYY-MM-DD".into()))?;
    let end = chrono::NaiveDate::parse_from_str(&input.end_date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Invalid end_date format, use YYYY-MM-DD".into()))?;

    if end <= start {
        return Err(AppError::Validation(
            "end_date must be after start_date".into(),
        ));
    }

    // Max 18 months for a fiscal year
    let months = (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
    if months > 18 {
        return Err(AppError::Validation(
            "Fiscal year cannot exceed 18 months".into(),
        ));
    }

    let fy = FiscalYear::new(&company_id, &input);

    sqlx::query(
        "INSERT INTO fiscal_years (id, company_id, start_date, end_date, is_closed, created_at)
         VALUES (?, ?, ?, ?, 0, ?)",
    )
    .bind(&fy.id)
    .bind(&fy.company_id)
    .bind(&fy.start_date)
    .bind(&fy.end_date)
    .bind(&fy.created_at)
    .execute(&state.pool)
    .await?;

    Ok(Json(fy))
}

async fn list_fiscal_years(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(company_id): Path<String>,
) -> Result<Json<Vec<FiscalYear>>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "viewer").await?;

    let years = sqlx::query_as::<_, FiscalYear>(
        "SELECT * FROM fiscal_years WHERE company_id = ? ORDER BY start_date DESC",
    )
    .bind(&company_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(years))
}

async fn get_fiscal_year(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<FiscalYear>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &id, "viewer").await?;

    let fy = sqlx::query_as::<_, FiscalYear>("SELECT * FROM fiscal_years WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Fiscal year {id} not found")))?;
    Ok(Json(fy))
}
