use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::money::Money;

/// Parameters for executing the annual closing.
#[derive(Debug, Deserialize)]
pub struct ClosingParams {
    /// Tax amount in SEK (can be adjusted from the estimated amount).
    pub tax_amount: Option<Money>,
    /// Whether to create opening balances for the next fiscal year.
    #[serde(default = "default_true")]
    pub carry_forward: bool,
}

fn default_true() -> bool {
    true
}

/// Result of executing the closing.
#[derive(Debug, Serialize)]
pub struct ClosingResult {
    pub closing_vouchers: Vec<ClosingVoucherInfo>,
    pub fiscal_year_closed: bool,
    pub next_fiscal_year_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClosingVoucherInfo {
    pub voucher_id: String,
    pub voucher_number: i32,
    pub description: String,
    pub total_amount: Money,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountBalance {
    account_number: i32,
    total_debit: Money,
    total_credit: Money,
}

/// Execute the annual closing for a fiscal year.
///
/// This creates closing vouchers (bokslutsverifikationer):
/// 1. Close revenue accounts (class 3) to 8999 Årets resultat
/// 2. Close expense accounts (class 4-7) to 8999 Årets resultat
/// 3. Close financial accounts (class 8, except 8999) to 8999 Årets resultat
/// 4. Book corporate tax (8910 → 2510)
/// 5. Transfer årets resultat to equity (8999 → 2099)
/// Then locks the fiscal year and optionally carries forward balances.
pub async fn execute_closing(
    pool: &SqlitePool,
    fiscal_year_id: &str,
    params: &ClosingParams,
) -> Result<ClosingResult, ClosingError> {
    // Fetch fiscal year
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await
    .map_err(ClosingError::Database)?;

    if fy.is_closed {
        return Err(ClosingError::AlreadyClosed);
    }

    // Fetch all account balances
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
    .await
    .map_err(ClosingError::Database)?;

    let mut tx = pool.begin().await.map_err(ClosingError::Database)?;
    let now = Utc::now().to_rfc3339();
    let mut closing_vouchers = Vec::new();

    // Collect income statement accounts to close (class 3-8, except 8999)
    let mut income_lines: Vec<(i32, Money, Money)> = Vec::new(); // (account, debit, credit)
    let mut net_result = Money::ZERO; // Running total of årets resultat

    for bal in &balances {
        let class = bal.account_number / 1000;
        if !(3..=8).contains(&class) {
            continue;
        }
        if bal.account_number == 8999 || bal.account_number == 8910 {
            continue; // Don't close tax or result accounts yet
        }

        let net = bal.total_debit - bal.total_credit;
        if net.is_zero() {
            continue;
        }

        // To close: reverse the balance
        // If account has debit balance (net > 0), credit it and debit 8999
        // If account has credit balance (net < 0), debit it and credit 8999
        if net.inner() > Decimal::ZERO {
            // Debit balance → credit to close
            income_lines.push((bal.account_number, Money::ZERO, net));
            net_result = net_result + net; // Debit 8999
        } else {
            // Credit balance → debit to close
            let abs_net = -net;
            income_lines.push((bal.account_number, abs_net, Money::ZERO));
            net_result = net_result - abs_net; // Credit 8999
        }
    }

    // Voucher 1: Close income/expense accounts to 8999
    if !income_lines.is_empty() {
        let voucher_num = next_voucher_number(&mut tx, fiscal_year_id).await?;
        let voucher_id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO vouchers (id, company_id, fiscal_year_id, voucher_number, date, description, is_closing_entry, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
        )
        .bind(&voucher_id)
        .bind(&fy.company_id)
        .bind(fiscal_year_id)
        .bind(voucher_num)
        .bind(&fy.end_date)
        .bind("Avslut resultaträkning")
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(ClosingError::Database)?;

        let mut total = Money::ZERO;
        for (account, debit, credit) in &income_lines {
            insert_voucher_line(&mut tx, &voucher_id, *account, *debit, *credit).await?;
            total = total + *debit + *credit;
        }

        // Balancing entry to 8999 Årets resultat
        if net_result.inner() > Decimal::ZERO {
            insert_voucher_line(&mut tx, &voucher_id, 8999, net_result, Money::ZERO).await?;
        } else if net_result.inner() < Decimal::ZERO {
            insert_voucher_line(&mut tx, &voucher_id, 8999, Money::ZERO, -net_result).await?;
        }

        closing_vouchers.push(ClosingVoucherInfo {
            voucher_id: voucher_id.clone(),
            voucher_number: voucher_num,
            description: "Avslut resultaträkning".into(),
            total_amount: total,
        });
    }

    // Voucher 2: Corporate tax
    // net_result is debit-side perspective: positive = expense > revenue (loss),
    // negative = revenue > expense (profit)
    // For tax: profit is when revenue > expenses, which means credit balance on 8999
    let taxable_result = -net_result; // Positive = profit
    let tax_amount = if let Some(t) = params.tax_amount {
        t
    } else if taxable_result.inner() > Decimal::ZERO {
        let rate: Decimal = "0.206".parse().unwrap();
        Money::new((taxable_result.inner() * rate).round_dp(0))
    } else {
        Money::ZERO
    };

    if !tax_amount.is_zero() {
        let voucher_num = next_voucher_number(&mut tx, fiscal_year_id).await?;
        let voucher_id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO vouchers (id, company_id, fiscal_year_id, voucher_number, date, description, is_closing_entry, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
        )
        .bind(&voucher_id)
        .bind(&fy.company_id)
        .bind(fiscal_year_id)
        .bind(voucher_num)
        .bind(&fy.end_date)
        .bind("Beräknad bolagsskatt")
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(ClosingError::Database)?;

        // Debit 8910 Skatt (expense), Credit 2510 Skatteskulder (liability)
        insert_voucher_line(&mut tx, &voucher_id, 8910, tax_amount, Money::ZERO).await?;
        insert_voucher_line(&mut tx, &voucher_id, 2510, Money::ZERO, tax_amount).await?;

        closing_vouchers.push(ClosingVoucherInfo {
            voucher_id: voucher_id.clone(),
            voucher_number: voucher_num,
            description: "Beräknad bolagsskatt".into(),
            total_amount: tax_amount,
        });
    }

    // Voucher 3: Close 8910 and 8999 → transfer to 2099 Årets resultat (equity)
    // After tax voucher, 8910 has a debit balance = tax_amount
    // 8999 has whatever balance from voucher 1
    // We need to close both to equity
    {
        let voucher_num = next_voucher_number(&mut tx, fiscal_year_id).await?;
        let voucher_id = Uuid::new_v4().to_string();
        let net_after_tax = taxable_result - tax_amount; // Profit after tax

        sqlx::query(
            "INSERT INTO vouchers (id, company_id, fiscal_year_id, voucher_number, date, description, is_closing_entry, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
        )
        .bind(&voucher_id)
        .bind(&fy.company_id)
        .bind(fiscal_year_id)
        .bind(voucher_num)
        .bind(&fy.end_date)
        .bind("Överföring årets resultat till eget kapital")
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(ClosingError::Database)?;

        // Close 8910 (has debit balance = tax_amount) → credit 8910
        if !tax_amount.is_zero() {
            insert_voucher_line(&mut tx, &voucher_id, 8910, Money::ZERO, tax_amount).await?;
        }

        // Close 8999: it has credit balance = taxable_result (profit) or debit balance (loss)
        // After closing income/expense: 8999 debit = net_result (positive if loss)
        // So 8999 credit balance = -net_result = taxable_result
        // To close: debit 8999 by taxable_result
        if taxable_result.inner() > Decimal::ZERO {
            insert_voucher_line(&mut tx, &voucher_id, 8999, taxable_result, Money::ZERO).await?;
        } else if taxable_result.inner() < Decimal::ZERO {
            insert_voucher_line(&mut tx, &voucher_id, 8999, Money::ZERO, -taxable_result).await?;
        }

        // Transfer net result to 2099 Årets resultat
        // Profit → credit 2099, Loss → debit 2099
        if net_after_tax.inner() > Decimal::ZERO {
            insert_voucher_line(&mut tx, &voucher_id, 2099, Money::ZERO, net_after_tax).await?;
        } else if net_after_tax.inner() < Decimal::ZERO {
            insert_voucher_line(&mut tx, &voucher_id, 2099, -net_after_tax, Money::ZERO).await?;
        }

        closing_vouchers.push(ClosingVoucherInfo {
            voucher_id,
            voucher_number: voucher_num,
            description: "Överföring årets resultat till eget kapital".into(),
            total_amount: net_after_tax.abs(),
        });
    }

    // Lock the fiscal year
    sqlx::query("UPDATE fiscal_years SET is_closed = 1, closed_at = ? WHERE id = ?")
        .bind(&now)
        .bind(fiscal_year_id)
        .execute(&mut *tx)
        .await
        .map_err(ClosingError::Database)?;

    // Carry forward opening balances to next fiscal year
    let mut next_fy_id = None;
    if params.carry_forward {
        next_fy_id = Some(
            carry_forward_balances(&mut tx, &fy.company_id, fiscal_year_id, &fy.end_date, &now)
                .await?,
        );
    }

    // Audit log
    crate::db::audit::log_action_tx(
        &mut tx,
        "fiscal_year",
        fiscal_year_id,
        "close",
        Some(&format!(
            "Closed with {} closing vouchers, tax: {}",
            closing_vouchers.len(),
            tax_amount
        )),
    )
    .await
    .ok();

    tx.commit().await.map_err(ClosingError::Database)?;

    Ok(ClosingResult {
        closing_vouchers,
        fiscal_year_closed: true,
        next_fiscal_year_id: next_fy_id,
    })
}

/// Carry forward balance sheet accounts (class 1-2) as opening balances for the next fiscal year.
async fn carry_forward_balances(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    company_id: &str,
    fiscal_year_id: &str,
    end_date: &str,
    now: &str,
) -> Result<String, ClosingError> {
    // Calculate the next fiscal year dates
    let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|e| ClosingError::Internal(format!("Invalid end date: {e}")))?;
    let next_start = end + chrono::Duration::days(1);
    let next_end = chrono::NaiveDate::from_ymd_opt(
        next_start.year_ce().1 as i32,
        end.month(),
        end.day(),
    )
    .unwrap_or(
        chrono::NaiveDate::from_ymd_opt(next_start.year_ce().1 as i32, 12, 31).unwrap(),
    );

    // Check if next fiscal year already exists
    let existing = sqlx::query_scalar::<_, String>(
        "SELECT id FROM fiscal_years WHERE company_id = ? AND start_date = ?",
    )
    .bind(company_id)
    .bind(next_start.to_string())
    .fetch_optional(&mut **tx)
    .await
    .map_err(ClosingError::Database)?;

    let next_fy_id = if let Some(id) = existing {
        id
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO fiscal_years (id, company_id, start_date, end_date, is_closed, created_at)
             VALUES (?, ?, ?, ?, 0, ?)",
        )
        .bind(&id)
        .bind(company_id)
        .bind(next_start.to_string())
        .bind(next_end.to_string())
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(ClosingError::Database)?;
        id
    };

    // Get final balances for all balance sheet accounts (class 1-2)
    // after all closing vouchers
    let final_balances = sqlx::query_as::<_, AccountBalance>(
        "SELECT vl.account_number,
                COALESCE(SUM(vl.debit), 0) as total_debit,
                COALESCE(SUM(vl.credit), 0) as total_credit
         FROM voucher_lines vl
         JOIN vouchers v ON vl.voucher_id = v.id
         WHERE v.fiscal_year_id = ?
         AND vl.account_number >= 1000 AND vl.account_number < 3000
         GROUP BY vl.account_number
         ORDER BY vl.account_number",
    )
    .bind(fiscal_year_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(ClosingError::Database)?;

    // Create opening balance voucher in the next fiscal year
    if !final_balances.is_empty() {
        let voucher_num =
            sqlx::query_scalar::<_, i32>(
                "SELECT COALESCE(MAX(voucher_number), 0) + 1 FROM vouchers WHERE fiscal_year_id = ?",
            )
            .bind(&next_fy_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(ClosingError::Database)?;

        let voucher_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO vouchers (id, company_id, fiscal_year_id, voucher_number, date, description, is_closing_entry, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
        )
        .bind(&voucher_id)
        .bind(company_id)
        .bind(&next_fy_id)
        .bind(voucher_num)
        .bind(next_start.to_string())
        .bind("Ingående balanser")
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(ClosingError::Database)?;

        for bal in &final_balances {
            let net = bal.total_debit - bal.total_credit;
            if net.is_zero() {
                continue;
            }
            if net.inner() > Decimal::ZERO {
                insert_voucher_line_tx(tx, &voucher_id, bal.account_number, net, Money::ZERO)
                    .await?;
            } else {
                insert_voucher_line_tx(tx, &voucher_id, bal.account_number, Money::ZERO, -net)
                    .await?;
            }
        }
    }

    Ok(next_fy_id)
}

async fn next_voucher_number(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    fiscal_year_id: &str,
) -> Result<i32, ClosingError> {
    sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(voucher_number), 0) + 1 FROM vouchers WHERE fiscal_year_id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(ClosingError::Database)
}

async fn insert_voucher_line(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    voucher_id: &str,
    account_number: i32,
    debit: Money,
    credit: Money,
) -> Result<(), ClosingError> {
    insert_voucher_line_tx(tx, voucher_id, account_number, debit, credit).await
}

async fn insert_voucher_line_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    voucher_id: &str,
    account_number: i32,
    debit: Money,
    credit: Money,
) -> Result<(), ClosingError> {
    let line_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO voucher_lines (id, voucher_id, account_number, debit, credit, description)
         VALUES (?, ?, ?, ?, ?, NULL)",
    )
    .bind(&line_id)
    .bind(voucher_id)
    .bind(account_number)
    .bind(debit)
    .bind(credit)
    .execute(&mut **tx)
    .await
    .map_err(ClosingError::Database)?;
    Ok(())
}

use chrono::Datelike;

#[derive(Debug, thiserror::Error)]
pub enum ClosingError {
    #[error("Fiscal year is already closed")]
    AlreadyClosed,
    #[error("Database error: {0}")]
    Database(sqlx::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<ClosingError> for crate::error::AppError {
    fn from(e: ClosingError) -> Self {
        match e {
            ClosingError::AlreadyClosed => crate::error::AppError::FiscalYearClosed,
            ClosingError::Database(e) => crate::error::AppError::Database(e),
            ClosingError::Internal(msg) => crate::error::AppError::Internal(msg),
        }
    }
}
