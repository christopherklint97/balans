use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub org_number: String,
    pub company_form: String,
    pub fiscal_year_start_month: i32,
    pub address: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCompany {
    pub name: String,
    pub org_number: String,
    /// One of: AB, HB, KB, EF, EK
    pub company_form: String,
    /// Month when fiscal year starts (1 = January)
    #[serde(default = "default_fy_start")]
    pub fiscal_year_start_month: i32,
    pub address: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
}

fn default_fy_start() -> i32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompany {
    pub name: Option<String>,
    pub org_number: Option<String>,
    pub address: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
}

impl Company {
    pub fn new(input: &CreateCompany) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name: input.name.clone(),
            org_number: input.org_number.replace('-', ""),
            company_form: input.company_form.clone(),
            fiscal_year_start_month: input.fiscal_year_start_month,
            address: input.address.clone(),
            postal_code: input.postal_code.clone(),
            city: input.city.clone(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
