use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::money::Money;

/// Result of a K2 compliance validation check.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub summary: ClosingSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
}

/// Summary of financial state before closing.
#[derive(Debug, Clone, Serialize)]
pub struct ClosingSummary {
    pub total_revenue: Money,
    pub total_expenses: Money,
    pub operating_result: Money,
    pub financial_income: Money,
    pub financial_expenses: Money,
    pub result_before_tax: Money,
    pub estimated_tax: Money,
    pub net_result: Money,
    pub total_assets: Money,
    pub total_equity_and_liabilities: Money,
    pub balance_difference: Money,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountBalance {
    account_number: i32,
    total_debit: Money,
    total_credit: Money,
}

/// Corporate tax rate for 2025-2026.
const CORPORATE_TAX_RATE: &str = "0.206";

/// Run K2 compliance validation for a fiscal year.
pub async fn validate_closing(
    pool: &SqlitePool,
    fiscal_year_id: &str,
) -> Result<ValidationResult, sqlx::Error> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Fetch fiscal year
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    // Check: fiscal year must not already be closed
    if fy.is_closed {
        errors.push(ValidationIssue {
            code: "FY_ALREADY_CLOSED".into(),
            message: "Räkenskapsåret är redan stängt.".into(),
            severity: IssueSeverity::Error,
        });
    }

    // Fetch all account balances for this fiscal year
    let balances = sqlx::query_as::<_, AccountBalance>(
        "SELECT vl.account_number,
                COALESCE(SUM(vl.debit), 0) as total_debit,
                COALESCE(SUM(vl.credit), 0) as total_credit
         FROM voucher_lines vl
         JOIN vouchers v ON vl.voucher_id = v.id
         WHERE v.fiscal_year_id = ?
         GROUP BY vl.account_number
         ORDER BY vl.account_number",
    )
    .bind(fiscal_year_id)
    .fetch_all(pool)
    .await?;

    // Check: must have at least one voucher
    let voucher_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM vouchers WHERE fiscal_year_id = ? AND is_closing_entry = 0",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    if voucher_count == 0 {
        errors.push(ValidationIssue {
            code: "NO_VOUCHERS".into(),
            message: "Inga verifikationer finns för räkenskapsåret.".into(),
            severity: IssueSeverity::Error,
        });
    }

    // Check: all vouchers must be balanced (integrity check)
    let unbalanced = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM vouchers v
         WHERE v.fiscal_year_id = ?
         AND (SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM voucher_lines WHERE voucher_id = v.id) != 0",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    if unbalanced > 0 {
        errors.push(ValidationIssue {
            code: "UNBALANCED_VOUCHERS".into(),
            message: format!("{unbalanced} verifikation(er) är inte balanserade."),
            severity: IssueSeverity::Error,
        });
    }

    // Calculate financial summary by account class
    let mut revenue_3xxx = Money::ZERO;
    let mut cost_4xxx = Money::ZERO;
    let mut cost_5xxx_6xxx = Money::ZERO;
    let mut cost_7xxx = Money::ZERO;
    let mut financial_income = Money::ZERO;
    let mut financial_expense = Money::ZERO;
    let mut total_assets = Money::ZERO;
    let mut total_equity_liabilities = Money::ZERO;

    for bal in &balances {
        let net = bal.total_debit - bal.total_credit;
        let class = bal.account_number / 1000;

        match class {
            1 => total_assets = total_assets + net,
            2 => total_equity_liabilities = total_equity_liabilities + net,
            3 => revenue_3xxx = revenue_3xxx + net,
            4 => cost_4xxx = cost_4xxx + net,
            5 | 6 => cost_5xxx_6xxx = cost_5xxx_6xxx + net,
            7 => cost_7xxx = cost_7xxx + net,
            8 => {
                // 83xx = financial income, 84xx-89xx = financial expense/other
                let sub = bal.account_number / 100;
                match sub {
                    83 => financial_income = financial_income + net,
                    84..=87 => financial_expense = financial_expense + net,
                    88 => {} // Bokslutsdispositioner — skip in pre-closing
                    89 => {} // Skatt/resultat — skip in pre-closing
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // Revenue accounts have credit balances (negative net in debit-credit)
    // So negate to get positive revenue
    let total_revenue = -revenue_3xxx;
    let total_expenses = cost_4xxx + cost_5xxx_6xxx + cost_7xxx;
    let operating_result = total_revenue - total_expenses;

    // Financial income is credit-side (negate), expense is debit-side
    let fin_income = -financial_income;
    let fin_expense = financial_expense;
    let result_before_tax = operating_result + fin_income - fin_expense;

    // Estimated tax (only if profit)
    let tax_rate = CORPORATE_TAX_RATE.parse::<Decimal>().unwrap();
    let estimated_tax = if result_before_tax.inner() > Decimal::ZERO {
        Money::new((result_before_tax.inner() * tax_rate).round_dp(0))
    } else {
        Money::ZERO
    };
    let net_result = result_before_tax - estimated_tax;

    // Balance sheet check
    // Assets should equal equity + liabilities (equity is negative in debit-credit system)
    let balance_diff = total_assets + total_equity_liabilities;

    // Warning if balance sheet doesn't balance (common before closing entries)
    if !balance_diff.is_zero() {
        warnings.push(ValidationIssue {
            code: "BALANCE_SHEET_DIFF".into(),
            message: format!(
                "Balansräkningen balanserar inte (diff: {}). Detta är normalt innan bokslutsverifikationer.",
                balance_diff
            ),
            severity: IssueSeverity::Warning,
        });
    }

    // Warning: no revenue
    if total_revenue.is_zero() {
        warnings.push(ValidationIssue {
            code: "NO_REVENUE".into(),
            message: "Inga intäkter bokförda under räkenskapsåret.".into(),
            severity: IssueSeverity::Warning,
        });
    }

    let summary = ClosingSummary {
        total_revenue,
        total_expenses,
        operating_result,
        financial_income: fin_income,
        financial_expenses: fin_expense,
        result_before_tax,
        estimated_tax,
        net_result,
        total_assets,
        total_equity_and_liabilities: -total_equity_liabilities, // Show as positive
        balance_difference: balance_diff,
    };

    let passed = errors.is_empty();

    Ok(ValidationResult {
        passed,
        errors,
        warnings,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tax_calculation() {
        let rate: Decimal = "0.206".parse().unwrap();
        let profit: Decimal = "100000.00".parse().unwrap();
        let tax = (profit * rate).round_dp(0);
        assert_eq!(tax, "20600".parse::<Decimal>().unwrap());
    }

    #[test]
    fn test_tax_on_loss() {
        let rate: Decimal = "0.206".parse().unwrap();
        let loss: Decimal = "-50000.00".parse().unwrap();
        // No tax on losses
        let tax = if loss > Decimal::ZERO {
            (loss * rate).round_dp(0)
        } else {
            Decimal::ZERO
        };
        assert_eq!(tax, Decimal::ZERO);
    }
}
