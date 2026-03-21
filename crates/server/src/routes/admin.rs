use axum::{
    extract::{Extension, Path, State},
    routing::{get, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::auth::middleware::AuthUser;
use crate::config::AppState;
use crate::error::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_all_users))
        .route("/admin/users/pending", get(list_pending_users))
        .route("/admin/users/{id}/approve", put(approve_user))
        .route("/admin/users/{id}/reject", put(reject_user))
        .route("/admin/users/{id}/activate", put(activate_user))
        .route("/admin/users/{id}/deactivate", put(deactivate_user))
        .route("/admin/users/{id}/role", put(change_user_role))
        .route(
            "/admin/companies/{id}/users",
            get(list_company_users).post(add_company_user),
        )
        .route(
            "/admin/companies/{company_id}/users/{user_id}",
            put(change_company_role).delete(remove_company_user),
        )
        .route("/admin/config", get(get_config))
}

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if auth.0.role != "admin" {
        return Err(AppError::Forbidden(
            "Administratörsbehörighet krävs".into(),
        ));
    }
    Ok(())
}

// --- Response types ---

#[derive(Debug, Serialize, FromRow)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
struct CompanyUserRow {
    user_id: String,
    email: String,
    name: String,
    company_role: String,
    system_role: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeRoleInput {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct AddCompanyUserInput {
    pub user_id: String,
    pub role: String,
}

// --- Handlers ---

async fn list_all_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<AdminUser>>, AppError> {
    require_admin(&auth)?;

    let users = sqlx::query_as::<_, AdminUser>(
        "SELECT id, email, name, role, status, is_active, created_at, updated_at
         FROM users ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(users))
}

async fn list_pending_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<AdminUser>>, AppError> {
    require_admin(&auth)?;

    let users = sqlx::query_as::<_, AdminUser>(
        "SELECT id, email, name, role, status, is_active, created_at, updated_at
         FROM users WHERE status = 'pending' ORDER BY created_at ASC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(users))
}

async fn approve_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<AdminUser>, AppError> {
    require_admin(&auth)?;

    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE users SET status = 'approved', updated_at = ? WHERE id = ? AND status = 'pending'",
    )
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Användaren hittades inte eller är inte väntande".into(),
        ));
    }

    crate::db::audit::log_action(&state.pool, "user", &id, "approve", None)
        .await
        .ok();

    fetch_admin_user(&state.pool, &id).await
}

async fn reject_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<AdminUser>, AppError> {
    require_admin(&auth)?;

    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE users SET status = 'rejected', updated_at = ? WHERE id = ? AND status = 'pending'",
    )
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Användaren hittades inte eller är inte väntande".into(),
        ));
    }

    crate::db::audit::log_action(&state.pool, "user", &id, "reject", None)
        .await
        .ok();

    fetch_admin_user(&state.pool, &id).await
}

async fn activate_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<AdminUser>, AppError> {
    require_admin(&auth)?;

    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE users SET is_active = 1, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.pool)
        .await?;

    crate::db::audit::log_action(&state.pool, "user", &id, "activate", None)
        .await
        .ok();

    fetch_admin_user(&state.pool, &id).await
}

async fn deactivate_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<AdminUser>, AppError> {
    require_admin(&auth)?;

    // Prevent self-deactivation
    if auth.0.sub == id {
        return Err(AppError::Validation(
            "Du kan inte inaktivera ditt eget konto".into(),
        ));
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE users SET is_active = 0, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.pool)
        .await?;

    crate::db::audit::log_action(&state.pool, "user", &id, "deactivate", None)
        .await
        .ok();

    fetch_admin_user(&state.pool, &id).await
}

async fn change_user_role(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(input): Json<ChangeRoleInput>,
) -> Result<Json<AdminUser>, AppError> {
    require_admin(&auth)?;

    if !["admin", "user", "viewer"].contains(&input.role.as_str()) {
        return Err(AppError::Validation(
            "Rollen måste vara admin, user eller viewer".into(),
        ));
    }

    // Prevent removing own admin role
    if auth.0.sub == id && input.role != "admin" {
        return Err(AppError::Validation(
            "Du kan inte ta bort din egen administratörsroll".into(),
        ));
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE users SET role = ?, updated_at = ? WHERE id = ?")
        .bind(&input.role)
        .bind(&now)
        .bind(&id)
        .execute(&state.pool)
        .await?;

    crate::db::audit::log_action(
        &state.pool,
        "user",
        &id,
        "change_role",
        Some(&input.role),
    )
    .await
    .ok();

    fetch_admin_user(&state.pool, &id).await
}

async fn list_company_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(company_id): Path<String>,
) -> Result<Json<Vec<CompanyUserRow>>, AppError> {
    require_admin(&auth)?;

    let users = sqlx::query_as::<_, CompanyUserRow>(
        "SELECT uc.user_id, u.email, u.name, uc.role as company_role, u.role as system_role
         FROM user_companies uc
         JOIN users u ON u.id = uc.user_id
         WHERE uc.company_id = ?
         ORDER BY u.name",
    )
    .bind(&company_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(users))
}

async fn add_company_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(company_id): Path<String>,
    Json(input): Json<AddCompanyUserInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    if !["owner", "admin", "member", "viewer"].contains(&input.role.as_str()) {
        return Err(AppError::Validation(
            "Rollen måste vara owner, admin, member eller viewer".into(),
        ));
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO user_companies (user_id, company_id, role, created_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&input.user_id)
    .bind(&company_id)
    .bind(&input.role)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    crate::db::audit::log_action(
        &state.pool,
        "company",
        &company_id,
        "add_user",
        Some(&format!("user={} role={}", input.user_id, input.role)),
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn change_company_role(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((company_id, user_id)): Path<(String, String)>,
    Json(input): Json<ChangeRoleInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    if !["owner", "admin", "member", "viewer"].contains(&input.role.as_str()) {
        return Err(AppError::Validation(
            "Rollen måste vara owner, admin, member eller viewer".into(),
        ));
    }

    sqlx::query("UPDATE user_companies SET role = ? WHERE user_id = ? AND company_id = ?")
        .bind(&input.role)
        .bind(&user_id)
        .bind(&company_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn remove_company_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((company_id, user_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    sqlx::query("DELETE FROM user_companies WHERE user_id = ? AND company_id = ?")
        .bind(&user_id)
        .bind(&company_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn get_config(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;

    Ok(Json(serde_json::json!({
        "mode": state.config.mode,
        "fixed_company_id": state.config.fixed_company_id,
    })))
}

// --- Helpers ---

async fn fetch_admin_user(
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<Json<AdminUser>, AppError> {
    let user = sqlx::query_as::<_, AdminUser>(
        "SELECT id, email, name, role, status, is_active, created_at, updated_at
         FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Användaren hittades inte".into()))?;

    Ok(Json(user))
}
