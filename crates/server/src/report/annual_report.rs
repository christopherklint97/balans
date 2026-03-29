use serde::Serialize;
use sqlx::SqlitePool;

use super::balance_sheet::{build_balance_sheet, BalanceSheet};
use super::income_statement::{build_income_statement, IncomeStatement};
use super::notes::{build_notes, Notes};
use crate::money::Money;

/// Complete K2 Årsredovisning data structure.
#[derive(Debug, Clone, Serialize)]
pub struct AnnualReport {
    pub company: CompanyInfo,
    pub fiscal_year: FiscalYearInfo,
    pub directors_report: DirectorsReport,
    pub income_statement: IncomeStatement,
    pub balance_sheet: BalanceSheet,
    pub notes: Notes,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompanyInfo {
    pub name: String,
    pub org_number: String,
    pub company_form: String,
    pub address: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FiscalYearInfo {
    pub start_date: String,
    pub end_date: String,
    pub is_closed: bool,
}

/// Förvaltningsberättelse per K2 requirements.
#[derive(Debug, Clone, Serialize)]
pub struct DirectorsReport {
    /// Verksamhetens art och inriktning
    pub business_description: String,
    /// Viktiga händelser under räkenskapsåret
    pub important_events: String,
    /// Förväntad framtida utveckling
    pub future_outlook: String,
    /// Förslag till vinstdisposition (for AB only)
    pub profit_allocation: Option<ProfitAllocation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfitAllocation {
    pub result_for_year: Money,
    pub retained_earnings: Money,
    pub total_available: Money,
    /// Proposed: carry forward
    pub carry_forward: Money,
    /// Proposed: dividend
    pub dividend: Money,
}

/// Build a complete annual report.
pub async fn build_annual_report(
    pool: &SqlitePool,
    fiscal_year_id: &str,
) -> Result<AnnualReport, sqlx::Error> {
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    let company = sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM companies WHERE id = ?",
    )
    .bind(&fy.company_id)
    .fetch_one(pool)
    .await?;

    // Find previous fiscal year for comparative figures
    let previous_fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE company_id = ? AND end_date < ? ORDER BY end_date DESC LIMIT 1",
    )
    .bind(&fy.company_id)
    .bind(&fy.start_date)
    .fetch_optional(pool)
    .await?;

    let prev_fy_id = previous_fy.as_ref().map(|f| f.id.as_str());

    let income_statement = build_income_statement(pool, fiscal_year_id, prev_fy_id).await?;
    let balance_sheet = build_balance_sheet(pool, fiscal_year_id, prev_fy_id).await?;
    let notes = build_notes(pool, &fy.company_id, fiscal_year_id).await?;

    // Build förvaltningsberättelse
    let net_result = income_statement.current.net_result;

    let profit_allocation = if company.company_form == "AB" {
        // Get retained earnings from balance sheet (unrestricted equity minus current year result)
        let retained = balance_sheet.current.equity_and_liabilities.unrestricted_equity - net_result;
        let total = retained + net_result;
        Some(ProfitAllocation {
            result_for_year: net_result,
            retained_earnings: retained,
            total_available: total,
            carry_forward: total, // Default: carry forward everything
            dividend: Money::ZERO,
        })
    } else {
        None
    };

    // Check for user-edited texts
    let saved_texts = sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(
        "SELECT business_description, important_events, future_outlook FROM directors_report_texts WHERE fiscal_year_id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_optional(pool)
    .await?;

    let default_business = format!(
        "{} bedriver verksamhet med säte i {}.",
        company.name,
        company.city.as_deref().unwrap_or("Sverige")
    );
    let default_events = "Inga väsentliga händelser att rapportera.".to_string();
    let default_outlook = "Bolaget bedömer att verksamheten kommer att fortsätta i oförändrad omfattning.".to_string();

    let directors_report = DirectorsReport {
        business_description: saved_texts
            .as_ref()
            .and_then(|t| t.0.clone())
            .unwrap_or(default_business),
        important_events: saved_texts
            .as_ref()
            .and_then(|t| t.1.clone())
            .unwrap_or(default_events),
        future_outlook: saved_texts
            .as_ref()
            .and_then(|t| t.2.clone())
            .unwrap_or(default_outlook),
        profit_allocation,
    };

    Ok(AnnualReport {
        company: CompanyInfo {
            name: company.name,
            org_number: company.org_number,
            company_form: company.company_form,
            address: company.address,
            postal_code: company.postal_code,
            city: company.city,
        },
        fiscal_year: FiscalYearInfo {
            start_date: fy.start_date,
            end_date: fy.end_date,
            is_closed: fy.is_closed,
        },
        directors_report,
        income_statement,
        balance_sheet,
        notes,
    })
}
