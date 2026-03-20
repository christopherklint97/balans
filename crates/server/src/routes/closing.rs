use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::k2::{
    closing::{execute_closing, ClosingParams, ClosingResult},
    validation::{validate_closing, ValidationResult},
};

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route(
            "/fiscal-years/{fy_id}/closing/validate",
            get(validate_handler),
        )
        .route(
            "/fiscal-years/{fy_id}/closing/execute",
            post(execute_handler),
        )
        .route(
            "/fiscal-years/{fy_id}/closing/status",
            get(status_handler),
        )
}

async fn validate_handler(
    State(pool): State<SqlitePool>,
    Path(fy_id): Path<String>,
) -> Result<Json<ValidationResult>, AppError> {
    let result = validate_closing(&pool, &fy_id).await?;
    Ok(Json(result))
}

async fn execute_handler(
    State(pool): State<SqlitePool>,
    Path(fy_id): Path<String>,
    Json(params): Json<ClosingParams>,
) -> Result<Json<ClosingResult>, AppError> {
    // Run validation first
    let validation = validate_closing(&pool, &fy_id).await?;
    if !validation.passed {
        return Err(AppError::Validation(
            format!(
                "Bokslutsvalidering misslyckades: {}",
                validation
                    .errors
                    .iter()
                    .map(|e| e.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ));
    }

    let result = execute_closing(&pool, &fy_id, &params).await?;
    Ok(Json(result))
}

/// Get the closing status of a fiscal year.
async fn status_handler(
    State(pool): State<SqlitePool>,
    Path(fy_id): Path<String>,
) -> Result<Json<ClosingStatus>, AppError> {
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(&fy_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Fiscal year {fy_id} not found")))?;

    let closing_voucher_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM vouchers WHERE fiscal_year_id = ? AND is_closing_entry = 1",
    )
    .bind(&fy_id)
    .fetch_one(&pool)
    .await?;

    let total_voucher_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM vouchers WHERE fiscal_year_id = ?",
    )
    .bind(&fy_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(ClosingStatus {
        is_closed: fy.is_closed,
        closed_at: fy.closed_at,
        closing_voucher_count,
        total_voucher_count,
    }))
}

#[derive(Debug, serde::Serialize)]
struct ClosingStatus {
    is_closed: bool,
    closed_at: Option<String>,
    closing_voucher_count: i32,
    total_voucher_count: i32,
}
