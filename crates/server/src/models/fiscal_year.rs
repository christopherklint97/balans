use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FiscalYear {
    pub id: String,
    pub company_id: String,
    pub start_date: String,
    pub end_date: String,
    pub is_closed: bool,
    pub closed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFiscalYear {
    /// ISO date, e.g. "2025-01-01"
    pub start_date: String,
    /// ISO date, e.g. "2025-12-31"
    pub end_date: String,
}

impl FiscalYear {
    pub fn new(company_id: &str, input: &CreateFiscalYear) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            company_id: company_id.to_string(),
            start_date: input.start_date.clone(),
            end_date: input.end_date.clone(),
            is_closed: false,
            closed_at: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}
