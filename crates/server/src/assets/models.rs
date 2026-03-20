use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::money::Money;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FixedAsset {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
    pub asset_type: String,
    pub acquisition_date: String,
    pub acquisition_cost: Money,
    pub useful_life_months: i32,
    pub residual_value: Money,
    pub depreciation_start_date: String,
    pub asset_account: i32,
    pub depreciation_account: i32,
    pub expense_account: i32,
    pub is_disposed: bool,
    pub disposal_date: Option<String>,
    pub disposal_amount: Money,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFixedAsset {
    pub name: String,
    pub description: Option<String>,
    pub asset_type: String,
    pub acquisition_date: String,
    pub acquisition_cost: Money,
    pub useful_life_months: i32,
    #[serde(default)]
    pub residual_value: Money,
    /// Defaults to acquisition_date if not provided
    pub depreciation_start_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DisposeAsset {
    pub disposal_date: String,
    pub disposal_amount: Money,
}

/// Default account mappings per asset type.
pub fn default_accounts(asset_type: &str) -> (i32, i32, i32) {
    match asset_type {
        "intangible" => (1010, 1080, 7810),
        "building" => (1110, 1119, 7820),
        "machinery" => (1210, 1219, 7831),
        "equipment" => (1220, 1229, 7832),
        "vehicle" => (1240, 1249, 7833),
        "computer" => (1250, 1259, 7834),
        "financial" => (1310, 1310, 7810), // Financial assets typically not depreciated
        _ => (1220, 1229, 7832),
    }
}

/// K2 suggested useful life per asset type (in months).
pub fn suggested_useful_life(asset_type: &str) -> i32 {
    match asset_type {
        "intangible" => 60,  // Max 5 years for K2 (goodwill, patents)
        "building" => 600,   // 50 years
        "machinery" => 60,   // 5 years
        "equipment" => 60,   // 5 years
        "vehicle" => 72,     // 6 years
        "computer" => 36,    // 3 years
        "financial" => 0,    // Not depreciated
        _ => 60,
    }
}

/// Human-readable asset type labels.
pub fn asset_type_label(asset_type: &str) -> &'static str {
    match asset_type {
        "intangible" => "Immateriell tillgång",
        "building" => "Byggnad",
        "machinery" => "Maskin",
        "equipment" => "Inventarie",
        "vehicle" => "Fordon",
        "computer" => "Dator",
        "financial" => "Finansiell tillgång",
        _ => "Övrigt",
    }
}
