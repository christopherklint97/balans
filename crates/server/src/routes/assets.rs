use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use crate::config::AppState;
use uuid::Uuid;

use crate::access::verify_company_access;
use crate::assets::depreciation::{
    build_depreciation_summary, generate_depreciation_vouchers, DepreciationSummary,
};
use crate::assets::models::*;
use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::money::Money;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/companies/{company_id}/assets",
            get(list_assets).post(create_asset),
        )
        .route("/assets/{id}", get(get_asset))
        .route("/assets/{id}/dispose", post(dispose_asset))
        .route(
            "/companies/{company_id}/fiscal-years/{fy_id}/depreciation",
            get(depreciation_summary_handler),
        )
        .route(
            "/companies/{company_id}/fiscal-years/{fy_id}/depreciation/generate",
            post(generate_depreciation_handler),
        )
}

async fn list_assets(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(company_id): Path<String>,
) -> Result<Json<Vec<FixedAsset>>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "viewer").await?;
    let assets = sqlx::query_as::<_, FixedAsset>(
        "SELECT * FROM fixed_assets WHERE company_id = ? ORDER BY acquisition_date",
    )
    .bind(&company_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(assets))
}

async fn create_asset(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(company_id): Path<String>,
    Json(input): Json<CreateFixedAsset>,
) -> Result<Json<FixedAsset>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "member").await?;
    // Validate asset type
    let valid_types = [
        "intangible",
        "building",
        "machinery",
        "equipment",
        "vehicle",
        "computer",
        "financial",
    ];
    if !valid_types.contains(&input.asset_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid asset type: {}. Must be one of: {}",
            input.asset_type,
            valid_types.join(", ")
        )));
    }

    if input.acquisition_cost.to_ore() <= 0 {
        return Err(AppError::Validation(
            "Acquisition cost must be positive".into(),
        ));
    }

    if input.useful_life_months <= 0 && input.asset_type != "financial" {
        return Err(AppError::Validation(
            "Useful life must be positive".into(),
        ));
    }

    let (asset_acc, dep_acc, exp_acc) = default_accounts(&input.asset_type);
    let dep_start = input
        .depreciation_start_date
        .clone()
        .unwrap_or_else(|| input.acquisition_date.clone());

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO fixed_assets (id, company_id, name, description, asset_type,
         acquisition_date, acquisition_cost, useful_life_months, residual_value,
         depreciation_start_date, asset_account, depreciation_account, expense_account,
         is_disposed, disposal_amount, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?, ?)",
    )
    .bind(&id)
    .bind(&company_id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.asset_type)
    .bind(&input.acquisition_date)
    .bind(input.acquisition_cost)
    .bind(input.useful_life_months)
    .bind(input.residual_value)
    .bind(&dep_start)
    .bind(asset_acc)
    .bind(dep_acc)
    .bind(exp_acc)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    crate::db::audit::log_action(
        &state.pool,
        "asset",
        &id,
        "create",
        Some(&format!("{} ({}, {} SEK)", input.name, input.asset_type, input.acquisition_cost)),
    )
    .await
    .ok();

    let asset = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(asset))
}

async fn get_asset(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<FixedAsset>, AppError> {
    let asset = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Asset {id} not found")))?;
    verify_company_access(&state.pool, &auth.0.sub, &asset.company_id, "viewer").await?;
    Ok(Json(asset))
}

async fn dispose_asset(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(input): Json<DisposeAsset>,
) -> Result<Json<FixedAsset>, AppError> {
    let existing = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Asset {id} not found")))?;

    verify_company_access(&state.pool, &auth.0.sub, &existing.company_id, "member").await?;

    if existing.is_disposed {
        return Err(AppError::Validation("Asset already disposed".into()));
    }

    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE fixed_assets SET is_disposed = 1, disposal_date = ?, disposal_amount = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&input.disposal_date)
    .bind(input.disposal_amount)
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    crate::db::audit::log_action(
        &state.pool,
        "asset",
        &id,
        "dispose",
        Some(&format!("{} disposed for {}", existing.name, input.disposal_amount)),
    )
    .await
    .ok();

    let asset = sqlx::query_as::<_, FixedAsset>("SELECT * FROM fixed_assets WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(asset))
}

async fn depreciation_summary_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((company_id, fy_id)): Path<(String, String)>,
) -> Result<Json<DepreciationSummary>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "viewer").await?;
    let summary = build_depreciation_summary(&state.pool, &company_id, &fy_id).await?;
    Ok(Json(summary))
}

async fn generate_depreciation_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((company_id, fy_id)): Path<(String, String)>,
) -> Result<Json<GenerateResult>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "member").await?;
    let voucher_ids = generate_depreciation_vouchers(&state.pool, &company_id, &fy_id).await?;
    Ok(Json(GenerateResult {
        vouchers_created: voucher_ids.len(),
        voucher_ids,
    }))
}

#[derive(Debug, serde::Serialize)]
struct GenerateResult {
    vouchers_created: usize,
    voucher_ids: Vec<String>,
}
