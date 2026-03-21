use axum::{
    extract::{Extension, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use crate::config::{AppMode, AppState};
use uuid::Uuid;

use super::jwt::create_token;
use super::middleware::AuthUser;
use super::models::*;
use crate::error::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

/// Routes that require authentication.
pub fn authenticated_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(me))
        .route("/auth/users", get(list_users))
}

async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterInput>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate input
    if input.email.is_empty() || !input.email.contains('@') {
        return Err(AppError::Validation("Invalid email address".into()));
    }
    if input.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".into(),
        ));
    }
    if input.name.is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }

    // Check if email already exists
    let exists = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM users WHERE email = ?")
        .bind(&input.email)
        .fetch_one(&state.pool)
        .await?;

    if exists > 0 {
        return Err(AppError::Validation("Email already registered".into()));
    }

    // Hash password
    let password_hash =
        bcrypt::hash(&input.password, bcrypt::DEFAULT_COST).map_err(|e| {
            AppError::Internal(format!("Password hashing failed: {e}"))
        })?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // First user becomes admin and is auto-approved
    let user_count =
        sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM users")
            .fetch_one(&state.pool)
            .await?;

    let is_first_user = user_count == 0;
    let role = if is_first_user { "admin" } else { "user" };
    let status = if is_first_user { "approved" } else { "pending" };

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, status, is_active, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(&id)
    .bind(&input.email)
    .bind(&password_hash)
    .bind(&input.name)
    .bind(role)
    .bind(status)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    // In fixed mode, add user to the fixed company
    if let (AppMode::Fixed, Some(company_id)) = (&state.config.mode, &state.config.fixed_company_id) {
        let uc_role = if is_first_user { "owner" } else { "member" };
        sqlx::query(
            "INSERT OR IGNORE INTO user_companies (user_id, company_id, role, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(company_id)
        .bind(uc_role)
        .bind(&now)
        .execute(&state.pool)
        .await?;
    }

    crate::db::audit::log_action(&state.pool, "user", &id, "register", Some(&input.email))
        .await
        .ok();

    if is_first_user {
        // First user: auto-approved, return token
        let token = create_token(&id, &input.email, role)
            .map_err(|e| AppError::Internal(format!("Token creation failed: {e}")))?;

        Ok(Json(AuthResponse {
            token: Some(token),
            user: Some(UserInfo {
                id,
                email: input.email,
                name: input.name,
                role: role.to_string(),
            }),
            status: "approved".to_string(),
            message: None,
        }))
    } else {
        // Subsequent users: pending approval
        Ok(Json(AuthResponse {
            token: None,
            user: None,
            status: "pending".to_string(),
            message: Some("Registrering mottagen. Väntar på godkännande från administratör.".to_string()),
        }))
    }
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&input.email)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::Validation("Felaktig e-post eller lösenord".into()))?;

    let valid = bcrypt::verify(&input.password, &user.password_hash)
        .map_err(|e| AppError::Internal(format!("Password verification failed: {e}")))?;

    if !valid {
        return Err(AppError::Validation("Felaktig e-post eller lösenord".into()));
    }

    // Check account status
    if !user.is_active {
        return Err(AppError::Forbidden("Kontot har inaktiverats. Kontakta administratör.".into()));
    }

    match user.status.as_str() {
        "pending" => {
            return Ok(Json(AuthResponse {
                token: None,
                user: None,
                status: "pending".to_string(),
                message: Some("Ditt konto väntar på godkännande.".to_string()),
            }));
        }
        "rejected" => {
            return Err(AppError::Forbidden("Din registrering har nekats.".into()));
        }
        "approved" => {} // continue
        _ => {
            return Err(AppError::Internal("Invalid account status".into()));
        }
    }

    let token = create_token(&user.id, &user.email, &user.role)
        .map_err(|e| AppError::Internal(format!("Token creation failed: {e}")))?;

    crate::db::audit::log_action(&state.pool, "user", &user.id, "login", Some(&user.email))
        .await
        .ok();

    Ok(Json(AuthResponse {
        token: Some(token),
        user: Some(UserInfo::from(&user)),
        status: "approved".to_string(),
        message: None,
    }))
}

async fn me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<UserInfo>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&auth.0.sub)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(UserInfo::from(&user)))
}

async fn list_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<UserInfo>>, AppError> {
    if auth.0.role != "admin" {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY name")
        .fetch_all(&state.pool)
        .await?;

    Ok(Json(users.iter().map(UserInfo::from).collect()))
}
