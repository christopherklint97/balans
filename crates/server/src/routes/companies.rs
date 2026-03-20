use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::db::seed::seed_bas_accounts;
use crate::error::AppError;
use crate::models::company::{Company, CreateCompany, UpdateCompany};
use crate::validation::validate_organisationsnummer;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/companies", post(create_company).get(list_companies))
        .route("/companies/{id}", get(get_company).put(update_company))
}

async fn create_company(
    State(pool): State<SqlitePool>,
    Json(input): Json<CreateCompany>,
) -> Result<Json<Company>, AppError> {
    // Validate org number
    if !validate_organisationsnummer(&input.org_number) {
        return Err(AppError::Validation(
            "Invalid organisationsnummer".to_string(),
        ));
    }

    // Validate company form
    if !["AB", "HB", "KB", "EF", "EK"].contains(&input.company_form.as_str()) {
        return Err(AppError::Validation(
            "company_form must be one of: AB, HB, KB, EF, EK".to_string(),
        ));
    }

    if !(1..=12).contains(&input.fiscal_year_start_month) {
        return Err(AppError::Validation(
            "fiscal_year_start_month must be between 1 and 12".to_string(),
        ));
    }

    let company = Company::new(&input);

    sqlx::query(
        "INSERT INTO companies (id, name, org_number, company_form, fiscal_year_start_month, address, postal_code, city, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&company.id)
    .bind(&company.name)
    .bind(&company.org_number)
    .bind(&company.company_form)
    .bind(company.fiscal_year_start_month)
    .bind(&company.address)
    .bind(&company.postal_code)
    .bind(&company.city)
    .bind(&company.created_at)
    .bind(&company.updated_at)
    .execute(&pool)
    .await?;

    // Seed BAS kontoplan for the new company
    seed_bas_accounts(&pool, &company.id).await?;

    crate::db::audit::log_action(
        &pool,
        "company",
        &company.id,
        "create",
        Some(&format!("{} ({})", company.name, company.org_number)),
    )
    .await
    .ok();

    Ok(Json(company))
}

async fn list_companies(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Company>>, AppError> {
    let companies = sqlx::query_as::<_, Company>("SELECT * FROM companies ORDER BY name")
        .fetch_all(&pool)
        .await?;
    Ok(Json(companies))
}

async fn get_company(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Company>, AppError> {
    let company = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Company {id} not found")))?;
    Ok(Json(company))
}

async fn update_company(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCompany>,
) -> Result<Json<Company>, AppError> {
    let existing = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Company {id} not found")))?;

    let now = Utc::now().to_rfc3339();
    let name = input.name.unwrap_or(existing.name);
    let address = input.address.or(existing.address);
    let postal_code = input.postal_code.or(existing.postal_code);
    let city = input.city.or(existing.city);

    sqlx::query(
        "UPDATE companies SET name = ?, address = ?, postal_code = ?, city = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&name)
    .bind(&address)
    .bind(&postal_code)
    .bind(&city)
    .bind(&now)
    .bind(&id)
    .execute(&pool)
    .await?;

    let company = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = ?")
        .bind(&id)
        .fetch_one(&pool)
        .await?;

    Ok(Json(company))
}
