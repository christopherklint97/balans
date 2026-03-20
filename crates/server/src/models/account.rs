use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Account {
    pub id: String,
    pub company_id: String,
    pub number: i32,
    pub name: String,
    pub account_type: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccount {
    pub number: i32,
    pub name: String,
    /// Optional: derived from number if not provided
    pub account_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}
