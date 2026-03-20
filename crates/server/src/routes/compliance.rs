use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};

use crate::error::AppError;
use crate::k2::eligibility::{check_eligibility, EligibilityInput, EligibilityResult};
use crate::report::multi_year::{build_multi_year_overview, MultiYearOverview};

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route(
            "/companies/{company_id}/fiscal-years/{fy_id}/k2-eligibility",
            get(eligibility_handler),
        )
        .route(
            "/companies/{company_id}/multi-year",
            get(multi_year_handler),
        )
        .route(
            "/companies/{company_id}/audit-log",
            get(audit_log_handler),
        )
}

#[derive(Deserialize)]
struct EligibilityQuery {
    average_employees: Option<i32>,
}

async fn eligibility_handler(
    State(pool): State<SqlitePool>,
    Path((company_id, fy_id)): Path<(String, String)>,
    Query(query): Query<EligibilityQuery>,
) -> Result<Json<EligibilityResult>, AppError> {
    let input = EligibilityInput {
        average_employees: query.average_employees,
    };
    let result = check_eligibility(&pool, &company_id, &fy_id, &input).await?;
    Ok(Json(result))
}

async fn multi_year_handler(
    State(pool): State<SqlitePool>,
    Path(company_id): Path<String>,
) -> Result<Json<MultiYearOverview>, AppError> {
    let overview = build_multi_year_overview(&pool, &company_id).await?;
    Ok(Json(overview))
}

#[derive(Deserialize)]
struct AuditLogQuery {
    entity_type: Option<String>,
    limit: Option<i32>,
}

#[derive(Debug, serde::Serialize, FromRow)]
struct AuditEntry {
    id: String,
    entity_type: String,
    entity_id: String,
    action: String,
    details: Option<String>,
    created_at: String,
}

async fn audit_log_handler(
    State(pool): State<SqlitePool>,
    Path(company_id): Path<String>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditEntry>>, AppError> {
    let limit = query.limit.unwrap_or(100).min(1000);

    let entries = if let Some(ref entity_type) = query.entity_type {
        sqlx::query_as::<_, AuditEntry>(
            "SELECT a.* FROM audit_log a
             WHERE a.entity_type = ?
             AND (a.entity_id = ? OR a.entity_id IN (
                SELECT id FROM fiscal_years WHERE company_id = ?
                UNION SELECT id FROM vouchers WHERE company_id = ?
             ))
             ORDER BY a.created_at DESC LIMIT ?",
        )
        .bind(entity_type)
        .bind(&company_id)
        .bind(&company_id)
        .bind(&company_id)
        .bind(limit)
        .fetch_all(&pool)
        .await?
    } else {
        sqlx::query_as::<_, AuditEntry>(
            "SELECT * FROM audit_log
             WHERE entity_id = ?
                OR entity_id IN (
                    SELECT id FROM fiscal_years WHERE company_id = ?
                    UNION SELECT id FROM vouchers WHERE company_id = ?
                )
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(&company_id)
        .bind(&company_id)
        .bind(&company_id)
        .bind(limit)
        .fetch_all(&pool)
        .await?
    };

    Ok(Json(entries))
}
