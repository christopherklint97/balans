use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::BTreeMap;

use crate::money::Money;

/// INK2 (Inkomstdeklaration 2) data structure.
/// Maps BAS accounts via SRU codes to INK2 form fields.
#[derive(Debug, Clone, Serialize)]
pub struct Ink2Data {
    pub company_name: String,
    pub org_number: String,
    pub fiscal_year_start: String,
    pub fiscal_year_end: String,
    /// All SRU fields with their aggregated values
    pub fields: Vec<Ink2Field>,
    /// Grouped by section for display
    pub sections: Vec<Ink2Section>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Ink2Field {
    pub sru_code: String,
    pub label: String,
    pub amount: Money,
    /// Account numbers contributing to this field
    pub accounts: Vec<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Ink2Section {
    pub title: String,
    pub fields: Vec<Ink2Field>,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountSru {
    number: i32,
    name: String,
    sru_code: Option<String>,
    account_type: String,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountBalance {
    account_number: i32,
    total_debit: Money,
    total_credit: Money,
}

/// SRU code labels for INK2 fields.
fn sru_label(code: &str) -> &'static str {
    match code {
        // Balance sheet — Assets
        "7201" => "Balanserade utgifter för utvecklingsarbeten",
        "7202" => "Koncessioner, patent, licenser, varumärken",
        "7203" => "Goodwill",
        "7207" => "Pågående arbeten, immateriella",
        "7210" => "Byggnader och mark",
        "7214" => "Maskiner och andra tekniska anläggningar",
        "7215" => "Inventarier, verktyg och installationer",
        "7220" => "Andelar i koncernföretag",
        "7225" => "Andra långfristiga fordringar",
        "7230" => "Varulager m.m.",
        "7234" => "Pågående arbeten för annans räkning",
        "7240" => "Kundfordringar",
        "7243" => "Fordringar hos anställda",
        "7248" => "Skattefordringar",
        "7249" => "Förutbetalda kostnader och upplupna intäkter",
        "7250" => "Kortfristiga placeringar",
        "7260" => "Kassa och bank",
        // Balance sheet — Equity & Liabilities
        "7300" => "Aktiekapital",
        "7302" => "Överkursfond",
        "7303" => "Reservfond",
        "7310" => "Balanserat resultat",
        "7312" => "Årets resultat",
        "7320" => "Periodiseringsfonder",
        "7321" => "Ackumulerade överavskrivningar",
        "7340" => "Skulder till kreditinstitut, lång",
        "7349" => "Övriga långfristiga skulder",
        "7360" => "Leverantörsskulder",
        "7361" => "Kortfristig del av långfristiga skulder",
        "7362" => "Skatteskulder",
        "7363" => "Moms och särskilda punktskatter",
        "7365" => "Personalens skatter, avgifter",
        "7368" => "Upplupna kostnader och förutbetalda intäkter",
        "7369" => "Övriga kortfristiga skulder",
        // Income statement
        "7410" => "Nettoomsättning",
        "7412" => "Övriga rörelseintäkter",
        "7420" => "Råvaror och förnödenheter, handelsvaror",
        "7430" => "Övriga externa kostnader",
        "7440" => "Personalkostnader — löner",
        "7441" => "Personalkostnader — sociala kostnader",
        "7450" => "Av- och nedskrivningar",
        "7510" => "Ränteintäkter och liknande",
        "7511" => "Räntekostnader och liknande",
        "7520" => "Förändring periodiseringsfonder",
        "7521" => "Förändring överavskrivningar",
        "7600" => "Skatt på årets resultat",
        _ => "Okänt fält",
    }
}

/// Section ordering for INK2 display.
fn sru_section(code: &str) -> &'static str {
    match code {
        "7201" | "7202" | "7203" | "7207" => "Immateriella anläggningstillgångar",
        "7210" | "7214" | "7215" => "Materiella anläggningstillgångar",
        "7220" | "7225" => "Finansiella anläggningstillgångar",
        "7230" | "7234" => "Varulager",
        "7240" | "7243" | "7248" | "7249" => "Kortfristiga fordringar",
        "7250" | "7260" => "Kassa, bank och placeringar",
        "7300" | "7302" | "7303" | "7310" | "7312" => "Eget kapital",
        "7320" | "7321" => "Obeskattade reserver",
        "7340" | "7349" => "Långfristiga skulder",
        "7360" | "7361" | "7362" | "7363" | "7365" | "7368" | "7369" => "Kortfristiga skulder",
        "7410" | "7412" => "Rörelseintäkter",
        "7420" | "7430" | "7440" | "7441" | "7450" => "Rörelsekostnader",
        "7510" | "7511" => "Finansiella poster",
        "7520" | "7521" => "Bokslutsdispositioner",
        "7600" => "Skatt",
        _ => "Övrigt",
    }
}

/// Build INK2 data from a fiscal year's account balances and SRU mappings.
pub async fn build_ink2(
    pool: &SqlitePool,
    company_id: &str,
    fiscal_year_id: &str,
) -> Result<Ink2Data, sqlx::Error> {
    let company = sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM companies WHERE id = ?",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await?;

    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    // Get all accounts with SRU codes
    let accounts = sqlx::query_as::<_, AccountSru>(
        "SELECT number, name, sru_code, account_type FROM accounts WHERE company_id = ? AND sru_code IS NOT NULL ORDER BY number",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await?;

    // Get balances (including closing entries for balance sheet, excluding for IS)
    let all_balances = sqlx::query_as::<_, AccountBalance>(
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

    let operational_balances = sqlx::query_as::<_, AccountBalance>(
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

    // Aggregate by SRU code
    let mut sru_amounts: BTreeMap<String, (Money, Vec<i32>)> = BTreeMap::new();

    for acc in &accounts {
        let sru = match &acc.sru_code {
            Some(s) => s.clone(),
            None => continue,
        };

        let class = acc.number / 1000;
        // Balance sheet accounts (1-2) use all_balances, IS accounts (3-8) use operational
        let balances = if class <= 2 { &all_balances } else { &operational_balances };

        let bal = balances
            .iter()
            .find(|b| b.account_number == acc.number);

        let amount = match bal {
            Some(b) => {
                // Assets (class 1): debit balance = positive
                // Equity/liabilities (class 2): credit balance = positive
                // Revenue (class 3): credit balance = positive
                // Expenses (class 4-8): debit balance = positive
                match class {
                    1 => b.total_debit - b.total_credit,
                    2 => b.total_credit - b.total_debit,
                    3 => b.total_credit - b.total_debit,
                    _ => b.total_debit - b.total_credit,
                }
            }
            None => Money::ZERO,
        };

        let entry = sru_amounts
            .entry(sru)
            .or_insert_with(|| (Money::ZERO, Vec::new()));
        entry.0 = entry.0 + amount;
        entry.1.push(acc.number);
    }

    // Build fields
    let fields: Vec<Ink2Field> = sru_amounts
        .iter()
        .filter(|(_, (amount, _))| !amount.is_zero())
        .map(|(sru, (amount, accs))| Ink2Field {
            sru_code: sru.clone(),
            label: sru_label(sru).to_string(),
            amount: *amount,
            accounts: accs.clone(),
        })
        .collect();

    // Group into sections
    let mut section_map: BTreeMap<String, Vec<Ink2Field>> = BTreeMap::new();
    for field in &fields {
        let section = sru_section(&field.sru_code).to_string();
        section_map
            .entry(section)
            .or_default()
            .push(field.clone());
    }

    let sections: Vec<Ink2Section> = section_map
        .into_iter()
        .map(|(title, fields)| Ink2Section { title, fields })
        .collect();

    Ok(Ink2Data {
        company_name: company.name,
        org_number: company.org_number,
        fiscal_year_start: fy.start_date,
        fiscal_year_end: fy.end_date,
        fields: fields.clone(),
        sections,
    })
}
