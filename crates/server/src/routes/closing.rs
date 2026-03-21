use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use crate::access::verify_fiscal_year_access;
use crate::auth::middleware::AuthUser;
use crate::config::AppState;

use crate::error::AppError;
use crate::k2::{
    closing::{execute_closing, ClosingParams, ClosingResult},
    validation::{validate_closing, ValidationResult},
};

pub fn routes() -> Router<AppState> {
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
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Json<ValidationResult>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let result = validate_closing(&state.pool, &fy_id).await?;
    Ok(Json(result))
}

async fn execute_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
    Json(params): Json<ClosingParams>,
) -> Result<Json<ClosingResult>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "member").await?;

    // Run validation first
    let validation = validate_closing(&state.pool, &fy_id).await?;
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

    let result = execute_closing(&state.pool, &fy_id, &params).await?;
    Ok(Json(result))
}

/// Get the closing status of a fiscal year.
async fn status_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Json<ClosingStatus>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(&fy_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Fiscal year {fy_id} not found")))?;

    let closing_voucher_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM vouchers WHERE fiscal_year_id = ? AND is_closing_entry = 1",
    )
    .bind(&fy_id)
    .fetch_one(&state.pool)
    .await?;

    let total_voucher_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM vouchers WHERE fiscal_year_id = ?",
    )
    .bind(&fy_id)
    .fetch_one(&state.pool)
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
