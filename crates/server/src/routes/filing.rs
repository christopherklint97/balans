use axum::{
    body::Body,
    extract::{Extension, Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use crate::access::verify_fiscal_year_access;
use crate::auth::middleware::AuthUser;
use crate::config::AppState;

use crate::error::AppError;
use crate::filing::{
    api::{BolagsverketClient, FilingResult},
    ixbrl::{compute_checksum, generate_ixbrl},
};
use crate::report::annual_report::build_annual_report;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/fiscal-years/{fy_id}/filing/ixbrl",
            get(download_ixbrl),
        )
        .route(
            "/fiscal-years/{fy_id}/filing/ixbrl/preview",
            get(preview_ixbrl),
        )
        .route(
            "/fiscal-years/{fy_id}/filing/submit",
            post(submit_filing),
        )
}

/// Preview the iXBRL document metadata without downloading.
async fn preview_ixbrl(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Json<IxbrlPreview>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let report = build_annual_report(&state.pool, &fy_id).await?;
    let ixbrl = generate_ixbrl(&report);
    let checksum = compute_checksum(&ixbrl);

    Ok(Json(IxbrlPreview {
        company_name: report.company.name,
        org_number: report.company.org_number,
        fiscal_year_start: report.fiscal_year.start_date,
        fiscal_year_end: report.fiscal_year.end_date,
        is_closed: report.fiscal_year.is_closed,
        document_size_bytes: ixbrl.len(),
        checksum_sha256: checksum,
    }))
}

/// Download the generated iXBRL file.
async fn download_ixbrl(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
) -> Result<Response, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "viewer").await?;

    let report = build_annual_report(&state.pool, &fy_id).await?;
    let ixbrl = generate_ixbrl(&report);

    let filename = format!(
        "arsredovisning_{}_{}.xhtml",
        report.company.org_number, report.fiscal_year.end_date
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xhtml+xml; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(ixbrl))
        .unwrap())
}

/// Submit the annual report to Bolagsverket.
async fn submit_filing(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(fy_id): Path<String>,
    Json(params): Json<SubmitParams>,
) -> Result<Json<FilingResult>, AppError> {
    verify_fiscal_year_access(&state.pool, &auth.0.sub, &fy_id, "member").await?;

    let report = build_annual_report(&state.pool, &fy_id).await?;

    // Verify the fiscal year is closed
    if !report.fiscal_year.is_closed {
        return Err(AppError::Validation(
            "Räkenskapsåret måste vara stängt innan årsredovisningen kan lämnas in.".into(),
        ));
    }

    let ixbrl = generate_ixbrl(&report);

    // Use test environment by default unless explicitly set to production
    let use_test = !params.production.unwrap_or(false);
    let client = BolagsverketClient::new(use_test);

    let result = client.file_annual_report(&ixbrl).await?;

    // Audit log
    crate::db::audit::log_action(
        &state.pool,
        "fiscal_year",
        &fy_id,
        "file_bolagsverket",
        Some(&format!(
            "Filed to {} — success: {}, ref: {}",
            if use_test { "TEST" } else { "PROD" },
            result.success,
            result.submission_reference.as_deref().unwrap_or("N/A")
        )),
    )
    .await
    .ok();

    Ok(Json(result))
}

#[derive(Debug, serde::Serialize)]
struct IxbrlPreview {
    company_name: String,
    org_number: String,
    fiscal_year_start: String,
    fiscal_year_end: String,
    is_closed: bool,
    document_size_bytes: usize,
    checksum_sha256: String,
}

#[derive(Debug, Deserialize)]
struct SubmitParams {
    /// Set to true to submit to Bolagsverket production (default: test environment)
    production: Option<bool>,
}
