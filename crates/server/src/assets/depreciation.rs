use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::models::FixedAsset;
use crate::money::Money;

/// Depreciation schedule for a single asset.
#[derive(Debug, Clone, Serialize)]
pub struct DepreciationSchedule {
    pub asset_id: String,
    pub asset_name: String,
    pub acquisition_cost: Money,
    pub residual_value: Money,
    pub depreciable_amount: Money,
    pub useful_life_months: i32,
    pub monthly_depreciation: Money,
    pub entries: Vec<DepreciationEntry>,
    pub total_depreciated: Money,
    pub book_value: Money,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepreciationEntry {
    pub period: String,
    pub amount: Money,
    pub accumulated: Money,
    pub book_value: Money,
}

/// Summary of depreciation for all assets in a fiscal year.
#[derive(Debug, Clone, Serialize)]
pub struct DepreciationSummary {
    pub fiscal_year_id: String,
    pub assets: Vec<AssetDepreciation>,
    pub total_depreciation: Money,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetDepreciation {
    pub asset_id: String,
    pub asset_name: String,
    pub asset_type: String,
    pub acquisition_cost: Money,
    pub depreciation_this_year: Money,
    pub accumulated_depreciation: Money,
    pub book_value: Money,
    pub expense_account: i32,
    pub depreciation_account: i32,
}

/// Calculate linear (straight-line) depreciation per K2 rules.
/// K2 requires linear depreciation unless another method better reflects usage.
pub fn calculate_linear_depreciation(
    acquisition_cost: Money,
    residual_value: Money,
    useful_life_months: i32,
) -> Money {
    if useful_life_months <= 0 {
        return Money::ZERO;
    }
    let depreciable = acquisition_cost - residual_value;
    if depreciable.to_ore() <= 0 {
        return Money::ZERO;
    }

    // Monthly depreciation = depreciable amount / useful life months
    let monthly_ore = depreciable.to_ore() / useful_life_months as i64;
    Money::from_ore(monthly_ore)
}

/// Calculate depreciation amount for a specific fiscal year period.
/// Handles partial first year (K2 half-year rule: assets acquired in H2
/// may use half-year depreciation).
pub fn depreciation_for_period(
    asset: &FixedAsset,
    period_start: &NaiveDate,
    period_end: &NaiveDate,
) -> Money {
    if asset.is_disposed {
        // If disposed before period, no depreciation
        if let Some(ref dd) = asset.disposal_date {
            if let Ok(d) = NaiveDate::parse_from_str(dd, "%Y-%m-%d") {
                if d < *period_start {
                    return Money::ZERO;
                }
            }
        }
    }

    let dep_start = NaiveDate::parse_from_str(&asset.depreciation_start_date, "%Y-%m-%d")
        .unwrap_or(*period_start);

    // If depreciation hasn't started yet
    if dep_start > *period_end {
        return Money::ZERO;
    }

    let monthly = calculate_linear_depreciation(
        asset.acquisition_cost,
        asset.residual_value,
        asset.useful_life_months,
    );

    if monthly.is_zero() {
        return Money::ZERO;
    }

    // Calculate months of depreciation within this period
    let effective_start = if dep_start > *period_start {
        dep_start
    } else {
        *period_start
    };

    // Check if fully depreciated before this period
    let total_depreciable = asset.acquisition_cost - asset.residual_value;
    let months_since_start = months_between(
        &dep_start,
        &effective_start,
    );
    let already_depreciated_ore = monthly.to_ore() * months_since_start as i64;
    if already_depreciated_ore >= total_depreciable.to_ore() {
        return Money::ZERO;
    }

    let effective_end = if let Some(ref dd) = asset.disposal_date {
        if let Ok(d) = NaiveDate::parse_from_str(dd, "%Y-%m-%d") {
            if d < *period_end { d } else { *period_end }
        } else {
            *period_end
        }
    } else {
        *period_end
    };

    let months_in_period = months_between(&effective_start, &effective_end);
    if months_in_period <= 0 {
        return Money::ZERO;
    }

    let period_amount_ore = monthly.to_ore() * months_in_period as i64;

    // Don't depreciate more than remaining book value
    let remaining = total_depreciable.to_ore() - already_depreciated_ore;
    let capped = period_amount_ore.min(remaining);

    Money::from_ore(capped.max(0))
}

/// Calculate months between two dates (inclusive of start month).
fn months_between(start: &NaiveDate, end: &NaiveDate) -> i32 {
    if end < start {
        return 0;
    }
    let years = end.year() - start.year();
    let months = end.month() as i32 - start.month() as i32;
    let total = years * 12 + months;
    // Add 1 because we include the end month
    (total + 1).max(0)
}

/// Build depreciation summary for all active assets in a fiscal year.
pub async fn build_depreciation_summary(
    pool: &SqlitePool,
    company_id: &str,
    fiscal_year_id: &str,
) -> Result<DepreciationSummary, sqlx::Error> {
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    let period_start = NaiveDate::parse_from_str(&fy.start_date, "%Y-%m-%d")
        .unwrap_or(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    let period_end = NaiveDate::parse_from_str(&fy.end_date, "%Y-%m-%d")
        .unwrap_or(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());

    let assets = sqlx::query_as::<_, FixedAsset>(
        "SELECT * FROM fixed_assets WHERE company_id = ? ORDER BY acquisition_date",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    let mut asset_deps = Vec::new();
    let mut total = Money::ZERO;

    for asset in &assets {
        let dep_this_year = depreciation_for_period(asset, &period_start, &period_end);

        // Calculate accumulated depreciation up to end of this period
        let dep_start = NaiveDate::parse_from_str(&asset.depreciation_start_date, "%Y-%m-%d")
            .unwrap_or(period_start);
        let monthly = calculate_linear_depreciation(
            asset.acquisition_cost,
            asset.residual_value,
            asset.useful_life_months,
        );
        let total_months = months_between(&dep_start, &period_end);
        let max_dep = (asset.acquisition_cost - asset.residual_value).to_ore();
        let accumulated_ore = (monthly.to_ore() * total_months as i64).min(max_dep).max(0);
        let accumulated = Money::from_ore(accumulated_ore);
        let book_value = asset.acquisition_cost - accumulated;

        total = total + dep_this_year;

        asset_deps.push(AssetDepreciation {
            asset_id: asset.id.clone(),
            asset_name: asset.name.clone(),
            asset_type: asset.asset_type.clone(),
            acquisition_cost: asset.acquisition_cost,
            depreciation_this_year: dep_this_year,
            accumulated_depreciation: accumulated,
            book_value,
            expense_account: asset.expense_account,
            depreciation_account: asset.depreciation_account,
        });
    }

    Ok(DepreciationSummary {
        fiscal_year_id: fiscal_year_id.to_string(),
        assets: asset_deps,
        total_depreciation: total,
    })
}

/// Generate depreciation vouchers for a fiscal year.
/// Returns the voucher IDs created.
pub async fn generate_depreciation_vouchers(
    pool: &SqlitePool,
    company_id: &str,
    fiscal_year_id: &str,
) -> Result<Vec<String>, crate::k2::closing::ClosingError> {
    use crate::k2::closing::ClosingError;

    let summary = build_depreciation_summary(pool, company_id, fiscal_year_id)
        .await
        .map_err(ClosingError::Database)?;

    if summary.total_depreciation.is_zero() {
        return Ok(Vec::new());
    }

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

    let mut tx = pool.begin().await.map_err(ClosingError::Database)?;
    let now = chrono::Utc::now().to_rfc3339();
    let mut voucher_ids = Vec::new();

    // Create one voucher per asset with depreciation
    for dep in &summary.assets {
        if dep.depreciation_this_year.is_zero() {
            continue;
        }

        let voucher_num = sqlx::query_scalar::<_, i32>(
            "SELECT COALESCE(MAX(voucher_number), 0) + 1 FROM vouchers WHERE fiscal_year_id = ?",
        )
        .bind(fiscal_year_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(ClosingError::Database)?;

        let voucher_id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO vouchers (id, company_id, fiscal_year_id, voucher_number, date, description, is_closing_entry, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(&voucher_id)
        .bind(company_id)
        .bind(fiscal_year_id)
        .bind(voucher_num)
        .bind(&fy.end_date)
        .bind(format!("Avskrivning {}", dep.asset_name))
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(ClosingError::Database)?;

        // Debit expense account, Credit accumulated depreciation account
        let line1_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO voucher_lines (id, voucher_id, account_number, debit, credit, description)
             VALUES (?, ?, ?, ?, 0, ?)",
        )
        .bind(&line1_id)
        .bind(&voucher_id)
        .bind(dep.expense_account)
        .bind(dep.depreciation_this_year)
        .bind(format!("Avskrivning {}", dep.asset_name).as_str())
        .execute(&mut *tx)
        .await
        .map_err(ClosingError::Database)?;

        let line2_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO voucher_lines (id, voucher_id, account_number, debit, credit, description)
             VALUES (?, ?, ?, 0, ?, ?)",
        )
        .bind(&line2_id)
        .bind(&voucher_id)
        .bind(dep.depreciation_account)
        .bind(dep.depreciation_this_year)
        .bind(format!("Avskrivning {}", dep.asset_name).as_str())
        .execute(&mut *tx)
        .await
        .map_err(ClosingError::Database)?;

        voucher_ids.push(voucher_id);
    }

    crate::db::audit::log_action_tx(
        &mut tx,
        "fiscal_year",
        fiscal_year_id,
        "depreciation",
        Some(&format!(
            "Generated {} depreciation vouchers, total: {}",
            voucher_ids.len(),
            summary.total_depreciation
        )),
    )
    .await
    .ok();

    tx.commit().await.map_err(ClosingError::Database)?;
    Ok(voucher_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_depreciation() {
        // 120,000 SEK over 60 months = 2,000 SEK/month
        let monthly = calculate_linear_depreciation(
            Money::from_ore(12_000_000),
            Money::ZERO,
            60,
        );
        assert_eq!(monthly.to_ore(), 200_000); // 2000.00 SEK
    }

    #[test]
    fn test_depreciation_with_residual() {
        // 100,000 cost - 10,000 residual = 90,000 depreciable over 36 months = 2,500/month
        let monthly = calculate_linear_depreciation(
            Money::from_ore(10_000_000),
            Money::from_ore(1_000_000),
            36,
        );
        assert_eq!(monthly.to_ore(), 250_000); // 2500.00 SEK
    }

    #[test]
    fn test_months_between() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        assert_eq!(months_between(&start, &end), 12);

        let start = NaiveDate::from_ymd_opt(2025, 7, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        assert_eq!(months_between(&start, &end), 6);
    }

    #[test]
    fn test_zero_useful_life() {
        let monthly = calculate_linear_depreciation(
            Money::from_ore(10_000_000),
            Money::ZERO,
            0,
        );
        assert!(monthly.is_zero());
    }
}
