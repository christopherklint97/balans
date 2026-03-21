use axum::{
    body::Body,
    extract::{Extension, Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Json, Router,
};
use crate::config::AppState;

use crate::access::verify_company_access;
use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::tax::ink2::{build_ink2, Ink2Data};
use crate::tax::sru::generate_sru;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/companies/{company_id}/fiscal-years/{fy_id}/ink2",
            get(ink2_data_handler),
        )
        .route(
            "/companies/{company_id}/fiscal-years/{fy_id}/ink2/sru",
            get(sru_download_handler),
        )
}

/// Get the INK2 form data as JSON.
async fn ink2_data_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((company_id, fy_id)): Path<(String, String)>,
) -> Result<Json<Ink2Data>, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "viewer").await?;
    let data = build_ink2(&state.pool, &company_id, &fy_id).await?;
    Ok(Json(data))
}

/// Download the SRU file for Skatteverket.
async fn sru_download_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((company_id, fy_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    verify_company_access(&state.pool, &auth.0.sub, &company_id, "viewer").await?;
    let data = build_ink2(&state.pool, &company_id, &fy_id).await?;
    let sru = generate_sru(&data);

    let filename = format!("ink2_{}_{}.sru", data.org_number, data.fiscal_year_end);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(sru))
        .unwrap())
}
