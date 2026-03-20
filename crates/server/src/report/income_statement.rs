use serde::Serialize;
use sqlx::SqlitePool;

use crate::money::Money;

/// K2 Resultaträkning — kostnadsslagsindelad (classified by nature of expense).
/// Per BFNAR 2016:10, chapter 4.
#[derive(Debug, Clone, Serialize)]
pub struct IncomeStatement {
    /// Current year amounts
    pub current: IncomeStatementData,
    /// Previous year amounts (comparative figures, required by K2)
    pub previous: Option<IncomeStatementData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncomeStatementData {
    pub fiscal_year: String,
    /// 1. Nettoomsättning
    pub net_revenue: Money,
    /// 2. Förändring av lager
    pub inventory_change: Money,
    /// 3. Aktiverat arbete för egen räkning
    pub capitalized_work: Money,
    /// 4. Övriga rörelseintäkter
    pub other_operating_income: Money,
    // --- Summa rörelseintäkter ---
    /// 5. Råvaror och förnödenheter
    pub raw_materials: Money,
    /// 6. Handelsvaror
    pub goods_for_resale: Money,
    /// 7. Övriga externa kostnader
    pub other_external_costs: Money,
    /// 8. Personalkostnader
    pub personnel_costs: Money,
    /// 9. Av- och nedskrivningar
    pub depreciation: Money,
    /// 10. Övriga rörelsekostnader
    pub other_operating_costs: Money,
    /// Rörelseresultat
    pub operating_result: Money,
    /// 11. Finansiella intäkter
    pub financial_income: Money,
    /// 12. Finansiella kostnader
    pub financial_costs: Money,
    /// Resultat efter finansiella poster
    pub result_after_financial: Money,
    /// 13. Bokslutsdispositioner
    pub appropriations: Money,
    /// Resultat före skatt
    pub result_before_tax: Money,
    /// 14. Skatt på årets resultat
    pub tax: Money,
    /// Årets resultat
    pub net_result: Money,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountBalance {
    account_number: i32,
    total_debit: Money,
    total_credit: Money,
}

/// Build the K2 income statement from voucher data.
pub async fn build_income_statement(
    pool: &SqlitePool,
    fiscal_year_id: &str,
    previous_fy_id: Option<&str>,
) -> Result<IncomeStatement, sqlx::Error> {
    let current = build_for_period(pool, fiscal_year_id).await?;

    let previous = if let Some(prev_id) = previous_fy_id {
        Some(build_for_period(pool, prev_id).await?)
    } else {
        None
    };

    Ok(IncomeStatement { current, previous })
}

async fn build_for_period(
    pool: &SqlitePool,
    fiscal_year_id: &str,
) -> Result<IncomeStatementData, sqlx::Error> {
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    // For the income statement, use non-closing vouchers for operational accounts,
    // but include the tax closing entry (8910) from closing vouchers.
    let balances = sqlx::query_as::<_, AccountBalance>(
        "SELECT vl.account_number,
                COALESCE(SUM(vl.debit), 0) as total_debit,
                COALESCE(SUM(vl.credit), 0) as total_credit
         FROM voucher_lines vl
         JOIN vouchers v ON vl.voucher_id = v.id
         WHERE v.fiscal_year_id = ?
           AND (v.is_closing_entry = 0 OR vl.account_number = 8910)
         GROUP BY vl.account_number",
    )
    .bind(fiscal_year_id)
    .fetch_all(pool)
    .await?;

    // Helper: sum net balance for account range (credit balance = positive for income)
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

    // Revenue accounts (class 3) — credit balances
    let net_revenue = net_credit(3000, 3099); // 30xx main revenue
    let inventory_change = net_credit(3100, 3199) + net_credit(4900, 4999); // 49xx lagerförändring
    let capitalized_work = net_credit(3800, 3899); // 38xx aktiverat arbete
    let other_operating_income = net_credit(3200, 3799) + net_credit(3900, 3999); // other 3xxx

    let total_operating_income =
        net_revenue + inventory_change + capitalized_work + other_operating_income;

    // Cost accounts — debit balances
    let raw_materials = net_debit(4000, 4099); // 40xx råvaror
    let goods_for_resale = net_debit(4100, 4899); // 41xx-48xx handelsvaror + other CoGS
    let other_external_costs = net_debit(5000, 6999); // 5xxx-6xxx
    let personnel_costs = net_debit(7000, 7699); // 70xx-76xx
    let depreciation = net_debit(7700, 7899); // 77xx-78xx avskrivningar
    let other_operating_costs = net_debit(7900, 7999); // 79xx

    let total_operating_costs = raw_materials
        + goods_for_resale
        + other_external_costs
        + personnel_costs
        + depreciation
        + other_operating_costs;

    let operating_result = total_operating_income - total_operating_costs;

    // Financial items (class 8)
    let financial_income = net_credit(8300, 8399); // 83xx
    let financial_costs = net_debit(8400, 8499); // 84xx

    let result_after_financial = operating_result + financial_income - financial_costs;

    // Appropriations (bokslutsdispositioner)
    let appropriations_income = net_credit(8800, 8899);
    let appropriations_expense = net_debit(8800, 8899);
    let appropriations = appropriations_income - appropriations_expense;

    let result_before_tax = result_after_financial + appropriations;

    // Tax
    let tax = net_debit(8900, 8949); // 89xx (not 8999)

    let net_result = result_before_tax - tax;

    let fiscal_label = format!("{} — {}", fy.start_date, fy.end_date);

    Ok(IncomeStatementData {
        fiscal_year: fiscal_label,
        net_revenue,
        inventory_change,
        capitalized_work,
        other_operating_income,
        raw_materials,
        goods_for_resale,
        other_external_costs,
        personnel_costs,
        depreciation,
        other_operating_costs,
        operating_result,
        financial_income,
        financial_costs,
        result_after_financial,
        appropriations,
        result_before_tax,
        tax,
        net_result,
    })
}
