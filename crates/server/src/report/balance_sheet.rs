use serde::Serialize;
use sqlx::SqlitePool;

use crate::money::Money;

/// K2 Balansräkning per BFNAR 2016:10, chapter 4.
#[derive(Debug, Clone, Serialize)]
pub struct BalanceSheet {
    pub current: BalanceSheetData,
    pub previous: Option<BalanceSheetData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BalanceSheetData {
    pub fiscal_year: String,
    // TILLGÅNGAR (Assets)
    pub assets: Assets,
    // EGET KAPITAL OCH SKULDER (Equity and liabilities)
    pub equity_and_liabilities: EquityAndLiabilities,
    pub total_assets: Money,
    pub total_equity_and_liabilities: Money,
}

#[derive(Debug, Clone, Serialize)]
pub struct Assets {
    // Anläggningstillgångar (Fixed assets)
    pub intangible_assets: Money,      // 10xx
    pub tangible_assets: Money,        // 11xx-12xx
    pub financial_fixed_assets: Money, // 13xx
    pub total_fixed_assets: Money,

    // Omsättningstillgångar (Current assets)
    pub inventory: Money,              // 14xx
    pub current_receivables: Money,    // 15xx-17xx
    pub short_term_investments: Money, // 18xx
    pub cash_and_bank: Money,          // 19xx
    pub total_current_assets: Money,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquityAndLiabilities {
    // Eget kapital (Equity)
    pub restricted_equity: Money,   // 2010-2019 (aktiekapital, reservfond)
    pub unrestricted_equity: Money, // 2080-2099 (balanserat resultat, årets resultat)
    pub total_equity: Money,

    // Obeskattade reserver (Untaxed reserves)
    pub untaxed_reserves: Money, // 21xx-22xx

    // Avsättningar (Provisions)
    pub provisions: Money, // 22xx (if used separately)

    // Långfristiga skulder (Long-term liabilities)
    pub long_term_liabilities: Money, // 23xx-24xx (>12 months)

    // Kortfristiga skulder (Current liabilities)
    pub current_liabilities: Money, // 24xx-29xx
    pub total_liabilities: Money,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountBalance {
    account_number: i32,
    total_debit: Money,
    total_credit: Money,
}

pub async fn build_balance_sheet(
    pool: &SqlitePool,
    fiscal_year_id: &str,
    previous_fy_id: Option<&str>,
) -> Result<BalanceSheet, sqlx::Error> {
    let current = build_for_period(pool, fiscal_year_id).await?;

    let previous = if let Some(prev_id) = previous_fy_id {
        Some(build_for_period(pool, prev_id).await?)
    } else {
        None
    };

    Ok(BalanceSheet { current, previous })
}

async fn build_for_period(
    pool: &SqlitePool,
    fiscal_year_id: &str,
) -> Result<BalanceSheetData, sqlx::Error> {
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    let balances = sqlx::query_as::<_, AccountBalance>(
        "SELECT vl.account_number,
                COALESCE(SUM(vl.debit), 0) as total_debit,
                COALESCE(SUM(vl.credit), 0) as total_credit
         FROM voucher_lines vl
         JOIN vouchers v ON vl.voucher_id = v.id
         WHERE v.fiscal_year_id = ?
         GROUP BY vl.account_number",
    )
    .bind(fiscal_year_id)
    .fetch_all(pool)
    .await?;

    // Assets = debit balance (debit - credit), positive = asset
    let net_debit = |from: i32, to: i32| -> Money {
        balances
            .iter()
            .filter(|b| b.account_number >= from && b.account_number <= to)
            .map(|b| b.total_debit - b.total_credit)
            .sum()
    };

    // Liabilities/equity = credit balance (credit - debit), positive = liability/equity
    let net_credit = |from: i32, to: i32| -> Money {
        balances
            .iter()
            .filter(|b| b.account_number >= from && b.account_number <= to)
            .map(|b| b.total_credit - b.total_debit)
            .sum()
    };

    // Assets
    let intangible_assets = net_debit(1000, 1099);
    let tangible_assets = net_debit(1100, 1299);
    let financial_fixed_assets = net_debit(1300, 1399);
    let total_fixed_assets = intangible_assets + tangible_assets + financial_fixed_assets;

    let inventory = net_debit(1400, 1499);
    let current_receivables = net_debit(1500, 1799);
    let short_term_investments = net_debit(1800, 1899);
    let cash_and_bank = net_debit(1900, 1999);
    let total_current_assets = inventory + current_receivables + short_term_investments + cash_and_bank;

    let total_assets = total_fixed_assets + total_current_assets;

    // Equity
    let restricted_equity = net_credit(2010, 2039); // Aktiekapital, överkursfond, reservfond
    let unrestricted_equity = net_credit(2080, 2099); // Balanserat resultat, årets resultat
    let total_equity = restricted_equity + unrestricted_equity;

    // Untaxed reserves
    let untaxed_reserves = net_credit(2100, 2199);

    // Provisions
    let provisions = net_credit(2200, 2299);

    // Long-term liabilities
    let long_term_liabilities = net_credit(2300, 2399);

    // Current liabilities
    let current_liabilities = net_credit(2400, 2999);

    let total_liabilities = long_term_liabilities + current_liabilities;
    let total_equity_and_liabilities =
        total_equity + untaxed_reserves + provisions + total_liabilities;

    let fiscal_label = format!("{}", fy.end_date);

    Ok(BalanceSheetData {
        fiscal_year: fiscal_label,
        assets: Assets {
            intangible_assets,
            tangible_assets,
            financial_fixed_assets,
            total_fixed_assets,
            inventory,
            current_receivables,
            short_term_investments,
            cash_and_bank,
            total_current_assets,
        },
        equity_and_liabilities: EquityAndLiabilities {
            restricted_equity,
            unrestricted_equity,
            total_equity,
            untaxed_reserves,
            provisions,
            long_term_liabilities,
            current_liabilities,
            total_liabilities,
        },
        total_assets,
        total_equity_and_liabilities,
    })
}
