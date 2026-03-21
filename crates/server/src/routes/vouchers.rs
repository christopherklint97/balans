use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use crate::access::verify_fiscal_year_access;
use crate::auth::middleware::AuthUser;
use crate::config::AppState;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::voucher::{CreateVoucher, Voucher, VoucherLine, VoucherWithLines};
use crate::money::Money;
use crate::validation::validate_bas_account_number;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/fiscal-years/{fy_id}/vouchers",
            post(create_voucher).get(list_vouchers),
        )
        .route("/vouchers/{id}", get(get_voucher).delete(delete_voucher))
}

async fn create_voucher(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
    Json(input): Json<CreateVoucher>,
) -> Result<Json<VoucherWithLines>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "member").await?;

    // Fetch fiscal year and verify it's not closed
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(&fy_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Fiscal year {fy_id} not found")))?;

    if fy.is_closed {
        return Err(AppError::FiscalYearClosed);
    }

    // Validate at least 2 lines
    if input.lines.len() < 2 {
        return Err(AppError::Validation(
            "A voucher must have at least 2 lines".into(),
        ));
    }

    // Validate voucher date is within fiscal year
    let voucher_date = chrono::NaiveDate::parse_from_str(&input.date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Invalid date format, use YYYY-MM-DD".into()))?;
    let fy_start = chrono::NaiveDate::parse_from_str(&fy.start_date, "%Y-%m-%d")
        .map_err(|_| AppError::Internal("Invalid fiscal year start date".into()))?;
    let fy_end = chrono::NaiveDate::parse_from_str(&fy.end_date, "%Y-%m-%d")
        .map_err(|_| AppError::Internal("Invalid fiscal year end date".into()))?;

    if voucher_date < fy_start || voucher_date > fy_end {
        return Err(AppError::Validation(format!(
            "Voucher date {voucher_date} is outside fiscal year ({} to {})",
            fy.start_date, fy.end_date
        )));
    }

    // Validate all account numbers
    for line in &input.lines {
        if !validate_bas_account_number(line.account_number) {
            return Err(AppError::Validation(format!(
                "Invalid account number: {}",
                line.account_number
            )));
        }

        // Verify account exists for this company
        let exists = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM accounts WHERE company_id = ? AND number = ?",
        )
        .bind(&fy.company_id)
        .bind(line.account_number)
        .fetch_one(&state.pool)
        .await?;

        if exists == 0 {
            return Err(AppError::Validation(format!(
                "Account {} not found for this company",
                line.account_number
            )));
        }
    }

    // Validate balance: total debits must equal total credits
    let total_debit: Money = input.lines.iter().map(|l| l.debit).sum();
    let total_credit: Money = input.lines.iter().map(|l| l.credit).sum();

    if total_debit != total_credit {
        return Err(AppError::UnbalancedVoucher {
            debit: total_debit.to_string(),
            credit: total_credit.to_string(),
        });
    }

    // Each line should have either debit or credit, not both
    for (i, line) in input.lines.iter().enumerate() {
        if !line.debit.is_zero() && !line.credit.is_zero() {
            return Err(AppError::Validation(format!(
                "Line {} has both debit and credit. Each line should use only one.",
                i + 1
            )));
        }
        if line.debit.is_zero() && line.credit.is_zero() {
            return Err(AppError::Validation(format!(
                "Line {} has zero debit and credit.",
                i + 1
            )));
        }
    }

    // Start transaction
    let mut tx = state.pool.begin().await?;

    // Get next voucher number
    let next_number = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(voucher_number), 0) + 1 FROM vouchers WHERE fiscal_year_id = ?",
    )
    .bind(&fy_id)
    .fetch_one(&mut *tx)
    .await?;

    let voucher_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO vouchers (id, company_id, fiscal_year_id, voucher_number, date, description, is_closing_entry, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 0, ?)"
    )
    .bind(&voucher_id)
    .bind(&fy.company_id)
    .bind(&fy_id)
    .bind(next_number)
    .bind(&input.date)
    .bind(&input.description)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    let mut lines = Vec::new();
    for line in &input.lines {
        let line_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO voucher_lines (id, voucher_id, account_number, debit, credit, description)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&line_id)
        .bind(&voucher_id)
        .bind(line.account_number)
        .bind(line.debit)
        .bind(line.credit)
        .bind(&line.description)
        .execute(&mut *tx)
        .await?;

        lines.push(VoucherLine {
            id: line_id,
            voucher_id: voucher_id.clone(),
            account_number: line.account_number,
            debit: line.debit,
            credit: line.credit,
            description: line.description.clone(),
        });
    }

    crate::db::audit::log_action_tx(
        &mut tx,
        "voucher",
        &voucher_id,
        "create",
        Some(&format!("#{} {} {}", next_number, input.date, input.description)),
    )
    .await
    .ok();

    tx.commit().await?;

    Ok(Json(VoucherWithLines {
        voucher: Voucher {
            id: voucher_id,
            company_id: fy.company_id,
            fiscal_year_id: fy_id,
            voucher_number: next_number,
            date: input.date,
            description: input.description,
            is_closing_entry: false,
            created_at: now,
        },
        lines,
    }))
}

async fn list_vouchers(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Json<Vec<Voucher>>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let vouchers = sqlx::query_as::<_, Voucher>(
        "SELECT * FROM vouchers WHERE fiscal_year_id = ? ORDER BY voucher_number",
    )
    .bind(&fy_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(vouchers))
}

async fn get_voucher(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<VoucherWithLines>, AppError> {
    let voucher = sqlx::query_as::<_, Voucher>("SELECT * FROM vouchers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Voucher {id} not found")))?;

    verify_fiscal_year_access(&state.pool, &auth.0.sub, &voucher.fiscal_year_id, "viewer").await?;

    let lines = sqlx::query_as::<_, VoucherLine>(
        "SELECT * FROM voucher_lines WHERE voucher_id = ? ORDER BY rowid",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(VoucherWithLines { voucher, lines }))
}

async fn delete_voucher(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let voucher = sqlx::query_as::<_, Voucher>("SELECT * FROM vouchers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Voucher {id} not found")))?;

    verify_fiscal_year_access(&state.pool, &auth.0.sub, &voucher.fiscal_year_id, "member").await?;

    // Check fiscal year is not closed
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(&voucher.fiscal_year_id)
    .fetch_one(&state.pool)
    .await?;

    if fy.is_closed {
        return Err(AppError::FiscalYearClosed);
    }

    // CASCADE will delete voucher_lines too
    sqlx::query("DELETE FROM vouchers WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
