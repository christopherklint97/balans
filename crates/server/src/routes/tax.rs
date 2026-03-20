use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Json, Router,
};
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::tax::ink2::{build_ink2, Ink2Data};
use crate::tax::sru::generate_sru;

pub fn routes() -> Router<SqlitePool> {
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
    State(pool): State<SqlitePool>,
    Path((company_id, fy_id)): Path<(String, String)>,
) -> Result<Json<Ink2Data>, AppError> {
    let data = build_ink2(&pool, &company_id, &fy_id).await?;
    Ok(Json(data))
}

/// Download the SRU file for Skatteverket.
async fn sru_download_handler(
    State(pool): State<SqlitePool>,
    Path((company_id, fy_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let data = build_ink2(&pool, &company_id, &fy_id).await?;
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
