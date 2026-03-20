use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use super::jwt::{validate_token, Claims};

/// Key for storing authenticated user claims in request extensions.
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

/// Middleware that requires a valid JWT token.
/// Extracts `Authorization: Bearer <token>` header.
pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, Response> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing or invalid Authorization header" })),
            )
                .into_response()
        })?;

    let claims = validate_token(token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid or expired token" })),
        )
            .into_response()
    })?;

    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}

/// Optional auth middleware — extracts user if token present, but doesn't reject.
pub async fn optional_auth(mut req: Request, next: Next) -> Response {
    if let Some(claims) = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| validate_token(token).ok())
    {
        req.extensions_mut().insert(AuthUser(claims));
    }
    next.run(req).await
}
