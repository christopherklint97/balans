use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::AppError;
use crate::money::Money;
use crate::sie::{
    parser::parse_sie,
    types::*,
    writer::{write_sie, SieType1Builder, SieType4Builder},
};
use crate::validation::account_type_from_number;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/companies/{company_id}/sie/preview", post(preview_import))
        .route("/companies/{company_id}/sie/import", post(import_sie))
        .route(
            "/fiscal-years/{fy_id}/sie/export/{sie_type}",
            get(export_sie),
        )
}

/// Upload a SIE file and get a preview of what will be imported.
async fn preview_import(
    State(_pool): State<SqlitePool>,
    Path(_company_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<SieImportPreview>, AppError> {
    let bytes = extract_file_bytes(&mut multipart).await?;
    let file = parse_sie(&bytes).map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(Json(SieImportPreview::from(&file)))
}

/// Import a SIE file into the database.
async fn import_sie(
    State(pool): State<SqlitePool>,
    Path(company_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>, AppError> {
    // Verify company exists
    sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM companies WHERE id = ?",
    )
    .bind(&company_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Company {company_id} not found")))?;

    let bytes = extract_file_bytes(&mut multipart).await?;
    let file = parse_sie(&bytes).map_err(|e| AppError::Validation(e.to_string()))?;

    let mut tx = pool.begin().await?;

    // Import accounts (merge with existing — skip duplicates)
    let mut accounts_imported = 0;
    for acc in &file.accounts {
        let exists = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM accounts WHERE company_id = ? AND number = ?",
        )
        .bind(&company_id)
        .bind(acc.number)
        .fetch_one(&mut *tx)
        .await?;

        if exists == 0 {
            let id = Uuid::new_v4().to_string();
            let account_type = acc
                .account_type
                .as_ref()
                .map(|t| sie_account_type_to_internal(t))
                .unwrap_or_else(|| account_type_from_number(acc.number).to_string());
            let now = chrono::Utc::now().to_rfc3339();

            sqlx::query(
                "INSERT INTO accounts (id, company_id, number, name, account_type, is_active, created_at)
                 VALUES (?, ?, ?, ?, ?, 1, ?)",
            )
            .bind(&id)
            .bind(&company_id)
            .bind(acc.number)
            .bind(&acc.name)
            .bind(&account_type)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
            accounts_imported += 1;
        }
    }

    // Find or create fiscal year for voucher import
    let mut fiscal_year_id = None;
    let mut vouchers_imported = 0;

    if let Some(fy) = file.fiscal_years.iter().find(|f| f.index == 0) {
        // Check if this fiscal year already exists
        let existing = sqlx::query_scalar::<_, String>(
            "SELECT id FROM fiscal_years WHERE company_id = ? AND start_date = ? AND end_date = ?",
        )
        .bind(&company_id)
        .bind(&fy.start_date)
        .bind(&fy.end_date)
        .fetch_optional(&mut *tx)
        .await?;

        let fy_id = if let Some(id) = existing {
            id
        } else {
            let id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO fiscal_years (id, company_id, start_date, end_date, is_closed, created_at)
                 VALUES (?, ?, ?, ?, 0, ?)",
            )
            .bind(&id)
            .bind(&company_id)
            .bind(&fy.start_date)
            .bind(&fy.end_date)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
            id
        };

        fiscal_year_id = Some(fy_id.clone());

        // Check fiscal year is not closed
        let is_closed = sqlx::query_scalar::<_, bool>(
            "SELECT is_closed FROM fiscal_years WHERE id = ?",
        )
        .bind(&fy_id)
        .fetch_one(&mut *tx)
        .await?;

        if is_closed {
            return Err(AppError::FiscalYearClosed);
        }

        // Import vouchers
        for sie_voucher in &file.vouchers {
            // Get next voucher number
            let next_num = sqlx::query_scalar::<_, i32>(
                "SELECT COALESCE(MAX(voucher_number), 0) + 1 FROM vouchers WHERE fiscal_year_id = ?",
            )
            .bind(&fy_id)
            .fetch_one(&mut *tx)
            .await?;

            let voucher_id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            sqlx::query(
                "INSERT INTO vouchers (id, company_id, fiscal_year_id, voucher_number, date, description, is_closing_entry, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, 0, ?)",
            )
            .bind(&voucher_id)
            .bind(&company_id)
            .bind(&fy_id)
            .bind(next_num)
            .bind(&sie_voucher.date)
            .bind(&sie_voucher.description)
            .bind(&now)
            .execute(&mut *tx)
            .await?;

            for trans in &sie_voucher.lines {
                let line_id = Uuid::new_v4().to_string();

                // SIE uses signed amounts: positive = debit, negative = credit
                let (debit_ore, credit_ore) = if trans.amount >= Decimal::ZERO {
                    (Money::new(trans.amount).to_ore(), 0i64)
                } else {
                    (0i64, Money::new(trans.amount.abs()).to_ore())
                };

                sqlx::query(
                    "INSERT INTO voucher_lines (id, voucher_id, account_number, debit, credit, description)
                     VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(&line_id)
                .bind(&voucher_id)
                .bind(trans.account_number)
                .bind(debit_ore)
                .bind(credit_ore)
                .bind(&trans.description)
                .execute(&mut *tx)
                .await?;
            }

            vouchers_imported += 1;
        }
    }

    tx.commit().await?;

    Ok(Json(ImportResult {
        accounts_imported,
        vouchers_imported,
        fiscal_year_id,
    }))
}

/// Export SIE file for a fiscal year.
async fn export_sie(
    State(pool): State<SqlitePool>,
    Path((fy_id, sie_type_str)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let sie_type = match sie_type_str.as_str() {
        "1" => SieType::Type1,
        "4" => SieType::Type4,
        _ => {
            return Err(AppError::Validation(
                "SIE type must be 1 or 4".to_string(),
            ))
        }
    };

    // Fetch fiscal year
    let fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(&fy_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Fiscal year {fy_id} not found")))?;

    // Fetch company
    let company = sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM companies WHERE id = ?",
    )
    .bind(&fy.company_id)
    .fetch_one(&pool)
    .await?;

    // Fetch accounts
    let accounts = sqlx::query_as::<_, crate::models::account::Account>(
        "SELECT * FROM accounts WHERE company_id = ? ORDER BY number",
    )
    .bind(&fy.company_id)
    .fetch_all(&pool)
    .await?;

    let account_tuples: Vec<(i32, String, Option<String>)> = accounts
        .iter()
        .map(|a| {
            let sie_type = internal_type_to_sie(&a.account_type);
            (a.number, a.name.clone(), Some(sie_type.to_string()))
        })
        .collect();

    // Calculate closing balances from voucher lines
    let balance_rows = sqlx::query_as::<_, BalanceRow>(
        "SELECT vl.account_number, COALESCE(SUM(vl.debit), 0) as total_debit, COALESCE(SUM(vl.credit), 0) as total_credit
         FROM voucher_lines vl
         JOIN vouchers v ON vl.voucher_id = v.id
         WHERE v.fiscal_year_id = ?
         GROUP BY vl.account_number
         ORDER BY vl.account_number",
    )
    .bind(&fy_id)
    .fetch_all(&pool)
    .await?;

    let closing_balances: Vec<(i32, Decimal)> = balance_rows
        .iter()
        .map(|r| {
            let balance = r.total_debit.inner() - r.total_credit.inner();
            (r.account_number, balance)
        })
        .collect();

    let sie_bytes = match sie_type {
        SieType::Type1 => {
            let builder = SieType1Builder {
                company_name: company.name,
                org_number: company.org_number,
                fiscal_year_start: fy.start_date,
                fiscal_year_end: fy.end_date,
                accounts: account_tuples,
                closing_balances,
            };
            write_sie(&builder.build())
        }
        SieType::Type4 => {
            // Fetch all vouchers with lines
            let vouchers = sqlx::query_as::<_, crate::models::voucher::Voucher>(
                "SELECT * FROM vouchers WHERE fiscal_year_id = ? ORDER BY voucher_number",
            )
            .bind(&fy_id)
            .fetch_all(&pool)
            .await?;

            let mut sie_vouchers = Vec::new();
            for v in &vouchers {
                let lines = sqlx::query_as::<_, crate::models::voucher::VoucherLine>(
                    "SELECT * FROM voucher_lines WHERE voucher_id = ? ORDER BY rowid",
                )
                .bind(&v.id)
                .fetch_all(&pool)
                .await?;

                let sie_lines: Vec<SieTransaction> = lines
                    .iter()
                    .map(|l| {
                        // Convert debit/credit back to signed amount
                        let amount = l.debit.inner() - l.credit.inner();
                        SieTransaction {
                            account_number: l.account_number,
                            amount,
                            date: None,
                            description: l.description.clone(),
                        }
                    })
                    .collect();

                sie_vouchers.push(SieVoucher {
                    series: "A".to_string(),
                    number: Some(v.voucher_number),
                    date: v.date.clone(),
                    description: v.description.clone(),
                    lines: sie_lines,
                });
            }

            let builder = SieType4Builder {
                company_name: company.name,
                org_number: company.org_number,
                fiscal_year_start: fy.start_date,
                fiscal_year_end: fy.end_date,
                accounts: account_tuples,
                opening_balances: Vec::new(), // TODO: carry from previous year
                closing_balances,
                vouchers: sie_vouchers,
            };
            write_sie(&builder.build())
        }
        _ => unreachable!(),
    };

    let filename = format!(
        "balans_sie{}_{}.se",
        sie_type.as_str(),
        chrono::Utc::now().format("%Y%m%d")
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(sie_bytes))
        .unwrap())
}

/// Extract file bytes from a multipart upload.
async fn extract_file_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        if field.name() == Some("file") {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("Failed to read file: {e}")))?;
            return Ok(bytes.to_vec());
        }
    }
    Err(AppError::Validation("No file field in upload".to_string()))
}

/// Convert SIE account type (T/S/K/I) to internal type.
fn sie_account_type_to_internal(sie_type: &str) -> String {
    match sie_type {
        "T" => "asset".to_string(),
        "S" => "liability".to_string(),
        "K" => "expense".to_string(),
        "I" => "revenue".to_string(),
        _ => "expense".to_string(),
    }
}

/// Convert internal account type to SIE type code.
fn internal_type_to_sie(internal: &str) -> &'static str {
    match internal {
        "asset" => "T",
        "equity" => "S",
        "liability" => "S",
        "revenue" => "I",
        "expense" => "K",
        _ => "K",
    }
}

#[derive(Debug, serde::Serialize)]
struct ImportResult {
    accounts_imported: usize,
    vouchers_imported: usize,
    fiscal_year_id: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct BalanceRow {
    account_number: i32,
    total_debit: Money,
    total_credit: Money,
}
