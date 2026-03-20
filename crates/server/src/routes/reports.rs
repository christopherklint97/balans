use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

use crate::error::AppError;
use crate::money::Money;

pub fn routes() -> Router<SqlitePool> {
    Router::new().route(
        "/fiscal-years/{fy_id}/trial-balance",
        get(trial_balance),
    )
}

#[derive(Debug, Serialize)]
pub struct TrialBalanceRow {
    pub account_number: i32,
    pub account_name: String,
    pub debit_total: Money,
    pub credit_total: Money,
    pub balance: Money,
}

#[derive(Debug, FromRow)]
struct RawTrialBalanceRow {
    account_number: i32,
    account_name: String,
    total_debit: Money,
    total_credit: Money,
}

async fn trial_balance(
    State(pool): State<SqlitePool>,
    Path(fy_id): Path<String>,
) -> Result<Json<Vec<TrialBalanceRow>>, AppError> {
    // Verify fiscal year exists
    let exists = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM fiscal_years WHERE id = ?")
        .bind(&fy_id)
        .fetch_one(&pool)
        .await?;
    if exists == 0 {
        return Err(AppError::NotFound(format!(
            "Fiscal year {fy_id} not found"
        )));
    }

    let rows = sqlx::query_as::<_, RawTrialBalanceRow>(
        "SELECT
            vl.account_number,
            a.name as account_name,
            COALESCE(SUM(vl.debit), 0) as total_debit,
            COALESCE(SUM(vl.credit), 0) as total_credit
         FROM voucher_lines vl
         JOIN vouchers v ON vl.voucher_id = v.id
         JOIN accounts a ON vl.account_number = a.number AND a.company_id = v.company_id
         WHERE v.fiscal_year_id = ?
         GROUP BY vl.account_number, a.name
         ORDER BY vl.account_number",
    )
    .bind(&fy_id)
    .fetch_all(&pool)
    .await?;

    let result: Vec<TrialBalanceRow> = rows
        .into_iter()
        .map(|r| TrialBalanceRow {
            account_number: r.account_number,
            account_name: r.account_name,
            debit_total: r.total_debit,
            credit_total: r.total_credit,
            balance: r.total_debit - r.total_credit,
        })
        .collect();

    Ok(Json(result))
}
