use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT claims embedded in the token.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Email
    pub email: String,
    /// Role
    pub role: String,
    /// Expiry (unix timestamp)
    pub exp: i64,
    /// Issued at (unix timestamp)
    pub iat: i64,
}

/// Get the JWT secret from the `JWT_SECRET` environment variable.
///
/// Panics if the variable is not set.
fn jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set")
        .into_bytes()
}

/// Token validity duration.
const TOKEN_EXPIRY_HOURS: i64 = 24;

/// Create a JWT token for a user.
pub fn create_token(user_id: &str, email: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: (now + Duration::hours(TOKEN_EXPIRY_HOURS)).timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_secret()),
    )
}

/// Validate and decode a JWT token.
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&jwt_secret()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_roundtrip() {
        let token = create_token("user-123", "test@example.com", "user").unwrap();
        let claims = validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_invalid_token() {
        let result = validate_token("invalid-token");
        assert!(result.is_err());
    }
}
