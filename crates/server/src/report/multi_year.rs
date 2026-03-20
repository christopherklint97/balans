use serde::Serialize;
use sqlx::SqlitePool;

use crate::money::Money;

/// Flerårsöversikt (multi-year overview) per K2.
/// Required for companies with >10 average employees.
#[derive(Debug, Clone, Serialize)]
pub struct MultiYearOverview {
    pub years: Vec<YearSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YearSummary {
    pub fiscal_year: String,
    pub net_revenue: Money,
    pub operating_result: Money,
    pub result_after_financial: Money,
    pub total_assets: Money,
    pub equity_ratio: String, // percentage
}

#[derive(Debug, sqlx::FromRow)]
struct AccountBalance {
    account_number: i32,
    total_debit: Money,
    total_credit: Money,
}

/// Build multi-year overview for all closed fiscal years.
pub async fn build_multi_year_overview(
    pool: &SqlitePool,
    company_id: &str,
) -> Result<MultiYearOverview, sqlx::Error> {
    let fiscal_years = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE company_id = ? ORDER BY start_date ASC",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut years = Vec::new();

    for fy in &fiscal_years {
        let balances = sqlx::query_as::<_, AccountBalance>(
            "SELECT vl.account_number,
                    COALESCE(SUM(vl.debit), 0) as total_debit,
                    COALESCE(SUM(vl.credit), 0) as total_credit
             FROM voucher_lines vl
             JOIN vouchers v ON vl.voucher_id = v.id
             WHERE v.fiscal_year_id = ? AND v.is_closing_entry = 0
             GROUP BY vl.account_number",
        )
        .bind(&fy.id)
        .fetch_all(pool)
        .await?;

        let net_credit = |from: i32, to: i32| -> Money {
            balances
                .iter()
                .filter(|b| b.account_number >= from && b.account_number <= to)
                .map(|b| b.total_credit - b.total_debit)
                .sum()
        };
        let net_debit = |from: i32, to: i32| -> Money {
            balances
                .iter()
                .filter(|b| b.account_number >= from && b.account_number <= to)
                .map(|b| b.total_debit - b.total_credit)
                .sum()
        };

        let net_revenue = net_credit(3000, 3099);
        let total_expenses = net_debit(4000, 7999);
        let operating_result = net_revenue - total_expenses;
        let financial_income = net_credit(8300, 8399);
        let financial_costs = net_debit(8400, 8499);
        let result_after_financial = operating_result + financial_income - financial_costs;

        // Total assets from all vouchers (including closing) for balance sheet
        let all_balances = sqlx::query_as::<_, AccountBalance>(
            "SELECT vl.account_number,
                    COALESCE(SUM(vl.debit), 0) as total_debit,
                    COALESCE(SUM(vl.credit), 0) as total_credit
             FROM voucher_lines vl
             JOIN vouchers v ON vl.voucher_id = v.id
             WHERE v.fiscal_year_id = ?
             GROUP BY vl.account_number",
        )
        .bind(&fy.id)
        .fetch_all(pool)
        .await?;

        let total_assets: Money = all_balances
            .iter()
            .filter(|b| b.account_number >= 1000 && b.account_number < 2000)
            .map(|b| b.total_debit - b.total_credit)
            .sum();

        let total_equity: Money = all_balances
            .iter()
            .filter(|b| b.account_number >= 2010 && b.account_number < 2100)
            .map(|b| b.total_credit - b.total_debit)
            .sum();

        let equity_ratio = if total_assets.inner() > rust_decimal::Decimal::ZERO {
            let ratio = (total_equity.inner() / total_assets.inner()
                * rust_decimal::Decimal::ONE_HUNDRED)
                .round_dp(1);
            format!("{ratio}%")
        } else {
            "N/A".to_string()
        };

        years.push(YearSummary {
            fiscal_year: format!("{} — {}", fy.start_date, fy.end_date),
            net_revenue,
            operating_result,
            result_after_financial,
            total_assets,
            equity_ratio,
        });
    }

    Ok(MultiYearOverview { years })
}
