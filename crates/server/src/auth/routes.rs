use axum::{
    extract::{Extension, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::jwt::create_token;
use super::middleware::AuthUser;
use super::models::*;
use crate::error::AppError;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

/// Routes that require authentication.
pub fn authenticated_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/auth/me", get(me))
        .route("/auth/users", get(list_users))
}

async fn register(
    State(pool): State<SqlitePool>,
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
        .fetch_one(&pool)
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

    // First user becomes admin
    let user_count =
        sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await?;
    let role = if user_count == 0 { "admin" } else { "user" };

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(&id)
    .bind(&input.email)
    .bind(&password_hash)
    .bind(&input.name)
    .bind(role)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await?;

    let token = create_token(&id, &input.email, role)
        .map_err(|e| AppError::Internal(format!("Token creation failed: {e}")))?;

    crate::db::audit::log_action(&pool, "user", &id, "register", Some(&input.email))
        .await
        .ok();

    Ok(Json(AuthResponse {
        token,
        user: UserInfo {
            id,
            email: input.email,
            name: input.name,
            role: role.to_string(),
        },
    }))
}

async fn login(
    State(pool): State<SqlitePool>,
    Json(input): Json<LoginInput>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ? AND is_active = 1")
        .bind(&input.email)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Validation("Invalid email or password".into()))?;

    let valid = bcrypt::verify(&input.password, &user.password_hash)
        .map_err(|e| AppError::Internal(format!("Password verification failed: {e}")))?;

    if !valid {
        return Err(AppError::Validation("Invalid email or password".into()));
    }

    let token = create_token(&user.id, &user.email, &user.role)
        .map_err(|e| AppError::Internal(format!("Token creation failed: {e}")))?;

    crate::db::audit::log_action(&pool, "user", &user.id, "login", Some(&user.email))
        .await
        .ok();

    Ok(Json(AuthResponse {
        token,
        user: UserInfo::from(&user),
    }))
}

async fn me(
    State(pool): State<SqlitePool>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<UserInfo>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&auth.0.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(UserInfo::from(&user)))
}

async fn list_users(
    State(pool): State<SqlitePool>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<UserInfo>>, AppError> {
    if auth.0.role != "admin" {
        return Err(AppError::Validation("Admin access required".into()));
    }

    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY name")
        .fetch_all(&pool)
        .await?;

    Ok(Json(users.iter().map(UserInfo::from).collect()))
}
