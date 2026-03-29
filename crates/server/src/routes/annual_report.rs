use axum::{
    body::Body,
    extract::{Extension, Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use crate::access::verify_fiscal_year_access;
use crate::auth::middleware::AuthUser;
use crate::config::AppState;

use crate::error::AppError;
use crate::report::annual_report::{build_annual_report, AnnualReport};
use crate::report::balance_sheet::{build_balance_sheet, BalanceSheet};
use crate::report::income_statement::{build_income_statement, IncomeStatement};
use crate::report::pdf::generate_pdf;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/fiscal-years/{fy_id}/income-statement",
            get(income_statement_handler),
        )
        .route(
            "/fiscal-years/{fy_id}/balance-sheet",
            get(balance_sheet_handler),
        )
        .route(
            "/fiscal-years/{fy_id}/annual-report",
            get(annual_report_handler),
        )
        .route(
            "/fiscal-years/{fy_id}/annual-report/pdf",
            get(annual_report_pdf_handler),
        )
        .route(
            "/fiscal-years/{fy_id}/directors-report-texts",
            get(get_directors_report_texts).put(put_directors_report_texts),
        )
}

async fn income_statement_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Json<IncomeStatement>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let prev_fy_id = find_previous_fy(&state.pool, &fy_id).await?;
    let is = build_income_statement(&state.pool, &fy_id, prev_fy_id.as_deref()).await?;
    Ok(Json(is))
}

async fn balance_sheet_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Json<BalanceSheet>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let prev_fy_id = find_previous_fy(&state.pool, &fy_id).await?;
    let bs = build_balance_sheet(&state.pool, &fy_id, prev_fy_id.as_deref()).await?;
    Ok(Json(bs))
}

async fn annual_report_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Json<AnnualReport>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let report = build_annual_report(&state.pool, &fy_id).await?;
    Ok(Json(report))
}

async fn annual_report_pdf_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Response, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let report = build_annual_report(&state.pool, &fy_id).await?;

    let pdf_bytes = generate_pdf(&report).map_err(|e| AppError::Internal(e))?;

    let filename = format!(
        "arsredovisning_{}_{}.pdf",
        report.company.org_number, report.fiscal_year.end_date
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(pdf_bytes))
        .unwrap())
}

#[derive(Debug, Deserialize)]
struct DirectorsReportTextsInput {
    business_description: Option<String>,
    important_events: Option<String>,
    future_outlook: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct DirectorsReportTextsOutput {
    business_description: Option<String>,
    important_events: Option<String>,
    future_outlook: Option<String>,
}

async fn get_directors_report_texts(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Json<DirectorsReportTextsOutput>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let row = sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(
        "SELECT business_description, important_events, future_outlook FROM directors_report_texts WHERE fiscal_year_id = ?",
    )
    .bind(&fy_id)
    .fetch_optional(&state.pool)
    .await?;

    let (bd, ie, fo) = row.unwrap_or((None, None, None));
    Ok(Json(DirectorsReportTextsOutput {
        business_description: bd,
        important_events: ie,
        future_outlook: fo,
    }))
}

async fn put_directors_report_texts(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
    Json(input): Json<DirectorsReportTextsInput>,
) -> Result<Json<DirectorsReportTextsOutput>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "member").await?;

    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO directors_report_texts (id, fiscal_year_id, business_description, important_events, future_outlook)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(fiscal_year_id) DO UPDATE SET
           business_description = excluded.business_description,
           important_events = excluded.important_events,
           future_outlook = excluded.future_outlook,
           updated_at = datetime('now')",
    )
    .bind(&id)
    .bind(&fy_id)
    .bind(&input.business_description)
    .bind(&input.important_events)
    .bind(&input.future_outlook)
    .execute(&state.pool)
    .await?;

    Ok(Json(DirectorsReportTextsOutput {
        business_description: input.business_description,
        important_events: input.important_events,
        future_outlook: input.future_outlook,
    }))
}

/// Find the previous fiscal year ID for comparative figures.
async fn find_previous_fy(
    pool: &SqlitePool,
    fiscal_year_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    let prev = sqlx::query_scalar::<_, String>(
        "SELECT id FROM fiscal_years WHERE company_id = ? AND end_date < ? ORDER BY end_date DESC LIMIT 1",
    )
    .bind(&fy.company_id)
    .bind(&fy.start_date)
    .fetch_optional(pool)
    .await?;

    Ok(prev)
}
