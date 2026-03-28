use axum::{
    body::Body,
    extract::{Extension, Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use uuid::Uuid;

use crate::access::verify_fiscal_year_access;
use crate::auth::middleware::AuthUser;
use crate::config::AppState;
use crate::error::AppError;
use crate::models::attachment::AttachmentMeta;
use crate::models::voucher::Voucher;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/vouchers/{voucher_id}/attachments",
            post(upload_attachment).get(list_attachments),
        )
        .route(
            "/vouchers/{voucher_id}/attachments/{id}",
            get(download_attachment).delete(delete_attachment),
        )
}

async fn get_voucher_and_verify(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    voucher_id: &str,
    min_role: &str,
) -> Result<Voucher, AppError> {
    let voucher = sqlx::query_as::<_, Voucher>("SELECT * FROM vouchers WHERE id = ?")
        .bind(voucher_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Voucher {voucher_id} not found")))?;

    verify_fiscal_year_access(pool, user_id, &voucher.fiscal_year_id, min_role).await?;
    Ok(voucher)
}

async fn upload_attachment(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(voucher_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<AttachmentMeta>, AppError> {
    get_voucher_and_verify(&state.pool, &auth.0.sub, &voucher_id, "member").await?;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("Failed to read multipart field: {e}"))
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }

        let filename = field
            .file_name()
            .unwrap_or("unknown")
            .to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let data = field.bytes().await.map_err(|e| {
            AppError::Validation(format!("Failed to read file data: {e}"))
        })?;

        if data.len() > MAX_FILE_SIZE {
            return Err(AppError::Validation(format!(
                "File too large: {} bytes (max {} bytes)",
                data.len(),
                MAX_FILE_SIZE
            )));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let size = data.len() as i64;

        sqlx::query(
            "INSERT INTO voucher_attachments (id, voucher_id, filename, content_type, size_bytes, data, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&voucher_id)
        .bind(&filename)
        .bind(&content_type)
        .bind(size)
        .bind(data.as_ref())
        .bind(&now)
        .execute(&state.pool)
        .await?;

        return Ok(Json(AttachmentMeta {
            id,
            voucher_id,
            filename,
            content_type,
            size_bytes: size,
            created_at: now,
        }));
    }

    Err(AppError::Validation("No file field found in request".into()))
}

async fn list_attachments(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(voucher_id): Path<String>,
) -> Result<Json<Vec<AttachmentMeta>>, AppError> {
    get_voucher_and_verify(&state.pool, &auth.0.sub, &voucher_id, "viewer").await?;

    let rows = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
        "SELECT id, voucher_id, filename, content_type, size_bytes, created_at
         FROM voucher_attachments WHERE voucher_id = ? ORDER BY created_at",
    )
    .bind(&voucher_id)
    .fetch_all(&state.pool)
    .await?;

    let metas = rows
        .into_iter()
        .map(|(id, voucher_id, filename, content_type, size_bytes, created_at)| AttachmentMeta {
            id,
            voucher_id,
            filename,
            content_type,
            size_bytes,
            created_at,
        })
        .collect();

    Ok(Json(metas))
}

async fn download_attachment(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((voucher_id, id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    get_voucher_and_verify(&state.pool, &auth.0.sub, &voucher_id, "viewer").await?;

    let row = sqlx::query_as::<_, (String, String, Vec<u8>)>(
        "SELECT filename, content_type, data FROM voucher_attachments WHERE id = ? AND voucher_id = ?",
    )
    .bind(&id)
    .bind(&voucher_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Attachment {id} not found")))?;

    let (filename, content_type, data) = row;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .unwrap())
}

async fn delete_attachment(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((voucher_id, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    get_voucher_and_verify(&state.pool, &auth.0.sub, &voucher_id, "member").await?;

    let result = sqlx::query("DELETE FROM voucher_attachments WHERE id = ? AND voucher_id = ?")
        .bind(&id)
        .bind(&voucher_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Attachment {id} not found")));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}
