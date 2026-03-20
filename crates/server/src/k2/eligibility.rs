use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::money::Money;

/// K2 eligibility thresholds per ÅRL.
/// A company may use K2 if it is a "smaller company", meaning it does NOT
/// exceed more than one of these thresholds for two consecutive fiscal years.
const MAX_EMPLOYEES: i32 = 50;
const MAX_BALANCE_SHEET_SEK: i64 = 40_000_000_00; // 40 MSEK in ören
const MAX_NET_REVENUE_SEK: i64 = 80_000_000_00; // 80 MSEK in ören

/// K2 eligibility check result.
#[derive(Debug, Clone, Serialize)]
pub struct EligibilityResult {
    pub is_eligible: bool,
    pub reason: Option<String>,
    pub checks: EligibilityChecks,
    pub thresholds: EligibilityThresholds,
}

#[derive(Debug, Clone, Serialize)]
pub struct EligibilityChecks {
    pub average_employees: i32,
    pub balance_sheet_total: Money,
    pub net_revenue: Money,
    pub employees_exceeded: bool,
    pub balance_exceeded: bool,
    pub revenue_exceeded: bool,
    pub thresholds_exceeded: i32,
    pub company_form_allowed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EligibilityThresholds {
    pub max_employees: i32,
    pub max_balance_sheet: Money,
    pub max_net_revenue: Money,
}

#[derive(Debug, Deserialize)]
pub struct EligibilityInput {
    pub average_employees: Option<i32>,
}

/// Check K2 eligibility for a company based on fiscal year data.
pub async fn check_eligibility(
    pool: &SqlitePool,
    company_id: &str,
    fiscal_year_id: &str,
    input: &EligibilityInput,
) -> Result<EligibilityResult, sqlx::Error> {
    let company = sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM companies WHERE id = ?",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await?;

    // Check company form
    let company_form_allowed = matches!(
        company.company_form.as_str(),
        "AB" | "HB" | "KB" | "EF" | "EK"
    );

    // Get balance sheet total and net revenue from voucher data
    let balance_total = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(vl.debit), 0)
         FROM voucher_lines vl
         JOIN vouchers v ON vl.voucher_id = v.id
         WHERE v.fiscal_year_id = ?
           AND vl.account_number >= 1000 AND vl.account_number < 2000",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    let revenue_total = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(vl.credit), 0)
         FROM voucher_lines vl
         JOIN vouchers v ON vl.voucher_id = v.id
         WHERE v.fiscal_year_id = ? AND v.is_closing_entry = 0
           AND vl.account_number >= 3000 AND vl.account_number < 3100",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    let avg_employees = input.average_employees.unwrap_or(0);

    let employees_exceeded = avg_employees > MAX_EMPLOYEES;
    let balance_exceeded = balance_total > MAX_BALANCE_SHEET_SEK;
    let revenue_exceeded = revenue_total > MAX_NET_REVENUE_SEK;

    let thresholds_exceeded = [employees_exceeded, balance_exceeded, revenue_exceeded]
        .iter()
        .filter(|&&x| x)
        .count() as i32;

    // K2 allowed if at most 1 threshold exceeded
    let is_eligible = company_form_allowed && thresholds_exceeded <= 1;

    let reason = if !company_form_allowed {
        Some(format!(
            "Företagsformen {} är inte tillåten för K2.",
            company.company_form
        ))
    } else if thresholds_exceeded > 1 {
        Some(format!(
            "Företaget överskrider {} av 3 gränsvärden. Högst 1 får överskridas.",
            thresholds_exceeded
        ))
    } else {
        None
    };

    Ok(EligibilityResult {
        is_eligible,
        reason,
        checks: EligibilityChecks {
            average_employees: avg_employees,
            balance_sheet_total: Money::from_ore(balance_total),
            net_revenue: Money::from_ore(revenue_total),
            employees_exceeded,
            balance_exceeded,
            revenue_exceeded,
            thresholds_exceeded,
            company_form_allowed,
        },
        thresholds: EligibilityThresholds {
            max_employees: MAX_EMPLOYEES,
            max_balance_sheet: Money::from_ore(MAX_BALANCE_SHEET_SEK),
            max_net_revenue: Money::from_ore(MAX_NET_REVENUE_SEK),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thresholds() {
        assert_eq!(MAX_EMPLOYEES, 50);
        assert_eq!(Money::from_ore(MAX_BALANCE_SHEET_SEK).to_string(), "40000000.00");
        assert_eq!(Money::from_ore(MAX_NET_REVENUE_SEK).to_string(), "80000000.00");
    }
}
