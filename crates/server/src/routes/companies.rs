use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use crate::config::AppState;

use crate::access::verify_company_access;
use crate::auth::middleware::AuthUser;
use crate::db::seed::seed_bas_accounts;
use crate::error::AppError;
use crate::models::company::{Company, CreateCompany, UpdateCompany};
use crate::validation::validate_organisationsnummer;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/companies", post(create_company).get(list_companies))
        .route("/companies/{id}", get(get_company).put(update_company))
}

async fn create_company(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
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
    .execute(&state.pool)
    .await?;

    // Auto-add creator as owner
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO user_companies (user_id, company_id, role, created_at)
         VALUES (?, ?, 'owner', ?)",
    )
    .bind(&auth.0.sub)
    .bind(&company.id)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    // Seed BAS kontoplan for the new company
    seed_bas_accounts(&state.pool, &company.id).await?;

    crate::db::audit::log_action(
        &state.pool,
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
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<Company>>, AppError> {
    // Admins see all companies; others see only their own
    let companies = if auth.0.role == "admin" {
        sqlx::query_as::<_, Company>("SELECT * FROM companies ORDER BY name")
            .fetch_all(&state.pool)
            .await?
    } else {
        sqlx::query_as::<_, Company>(
            "SELECT c.* FROM companies c
             JOIN user_companies uc ON uc.company_id = c.id
             WHERE uc.user_id = ?
             ORDER BY c.name",
        )
        .bind(&auth.0.sub)
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(companies))
}

async fn get_company(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Company>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &id, "viewer").await?;

    let company = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Company {id} not found")))?;
    Ok(Json(company))
}

async fn update_company(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCompany>,
) -> Result<Json<Company>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &id, "admin").await?;

    let existing = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Company {id} not found")))?;

    let now = Utc::now().to_rfc3339();
    let name = input.name.unwrap_or(existing.name);
    let org_number = if let Some(ref org) = input.org_number {
        if !validate_organisationsnummer(org) {
            return Err(AppError::Validation(
                "Invalid organisationsnummer".to_string(),
            ));
        }
        org.replace('-', "")
    } else {
        existing.org_number
    };
    let company_form = if let Some(ref form) = input.company_form {
        if !["AB", "HB", "KB", "EF", "EK"].contains(&form.as_str()) {
            return Err(AppError::Validation(
                "company_form must be one of: AB, HB, KB, EF, EK".to_string(),
            ));
        }
        form.clone()
    } else {
        existing.company_form
    };
    let address = input.address.or(existing.address);
    let postal_code = input.postal_code.or(existing.postal_code);
    let city = input.city.or(existing.city);

    sqlx::query(
        "UPDATE companies SET name = ?, org_number = ?, company_form = ?, address = ?, postal_code = ?, city = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&name)
    .bind(&org_number)
    .bind(&company_form)
    .bind(&address)
    .bind(&postal_code)
    .bind(&city)
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    let company = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(company))
}
