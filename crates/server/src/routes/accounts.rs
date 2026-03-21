use axum::{
    extract::{Extension, Path, Query, State},
    routing::{get, put},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use crate::config::AppState;
use uuid::Uuid;

use crate::access::verify_company_access;
use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::models::account::{Account, CreateAccount, UpdateAccount};
use crate::validation::{account_type_from_number, validate_bas_account_number};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/companies/{company_id}/accounts",
            get(list_accounts).post(create_account),
        )
        .route("/accounts/{id}", put(update_account))
}

#[derive(Deserialize)]
struct AccountFilter {
    account_type: Option<String>,
    active_only: Option<bool>,
}

async fn list_accounts(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(company_id): Path<String>,
    Query(filter): Query<AccountFilter>,
) -> Result<Json<Vec<Account>>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "viewer").await?;
    let mut query = String::from("SELECT * FROM accounts WHERE company_id = ?");
    if let Some(ref t) = filter.account_type {
        query.push_str(&format!(" AND account_type = '{t}'"));
    }
    if filter.active_only.unwrap_or(false) {
        query.push_str(" AND is_active = 1");
    }
    query.push_str(" ORDER BY number");

    let accounts = sqlx::query_as::<_, Account>(&query)
        .bind(&company_id)
        .fetch_all(&state.pool)
        .await?;

    Ok(Json(accounts))
}

async fn create_account(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(company_id): Path<String>,
    Json(input): Json<CreateAccount>,
) -> Result<Json<Account>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "member").await?;
    if !validate_bas_account_number(input.number) {
        return Err(AppError::Validation(
            "Account number must be between 1000 and 9999".into(),
        ));
    }

    let account_type = input
        .account_type
        .clone()
        .unwrap_or_else(|| account_type_from_number(input.number).to_string());

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO accounts (id, company_id, number, name, account_type, is_active, created_at)
         VALUES (?, ?, ?, ?, ?, 1, ?)",
    )
    .bind(&id)
    .bind(&company_id)
    .bind(input.number)
    .bind(&input.name)
    .bind(&account_type)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(account))
}

async fn update_account(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateAccount>,
) -> Result<Json<Account>, AppError> {
    let existing = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account {id} not found")))?;

    verify_company_access(&state.pool, &auth.0.sub, &existing.company_id, "member").await?;

    let name = input.name.unwrap_or(existing.name);
    let is_active = input.is_active.unwrap_or(existing.is_active);

    sqlx::query("UPDATE accounts SET name = ?, is_active = ? WHERE id = ?")
        .bind(&name)
        .bind(is_active)
        .bind(&id)
        .execute(&state.pool)
        .await?;

    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(account))
}
