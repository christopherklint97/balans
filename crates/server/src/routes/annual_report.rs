use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Json, Router,
};
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::report::annual_report::{build_annual_report, AnnualReport};
use crate::report::balance_sheet::{build_balance_sheet, BalanceSheet};
use crate::report::income_statement::{build_income_statement, IncomeStatement};
use crate::report::pdf::generate_pdf;

pub fn routes() -> Router<SqlitePool> {
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
}

async fn income_statement_handler(
    State(pool): State<SqlitePool>,
    Path(fy_id): Path<String>,
) -> Result<Json<IncomeStatement>, AppError> {
    let prev_fy_id = find_previous_fy(&pool, &fy_id).await?;
    let is = build_income_statement(&pool, &fy_id, prev_fy_id.as_deref()).await?;
    Ok(Json(is))
}

async fn balance_sheet_handler(
    State(pool): State<SqlitePool>,
    Path(fy_id): Path<String>,
) -> Result<Json<BalanceSheet>, AppError> {
    let prev_fy_id = find_previous_fy(&pool, &fy_id).await?;
    let bs = build_balance_sheet(&pool, &fy_id, prev_fy_id.as_deref()).await?;
    Ok(Json(bs))
}

async fn annual_report_handler(
    State(pool): State<SqlitePool>,
    Path(fy_id): Path<String>,
) -> Result<Json<AnnualReport>, AppError> {
    let report = build_annual_report(&pool, &fy_id).await?;
    Ok(Json(report))
}

async fn annual_report_pdf_handler(
    State(pool): State<SqlitePool>,
    Path(fy_id): Path<String>,
) -> Result<Response, AppError> {
    let report = build_annual_report(&pool, &fy_id).await?;

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
