use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Saas,
    Fixed,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mode: AppMode,
    /// Only used in Fixed mode — the single company ID users are assigned to.
    pub fixed_company_id: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let mode = match std::env::var("APP_MODE").unwrap_or_default().to_lowercase().as_str() {
            "fixed" => AppMode::Fixed,
            _ => AppMode::Saas,
        };

        let fixed_company_id = std::env::var("FIXED_COMPANY_ID").ok();

        if mode == AppMode::Fixed && fixed_company_id.is_none() {
            tracing::warn!("APP_MODE=fixed but FIXED_COMPANY_ID is not set");
        }

        Self {
            mode,
            fixed_company_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub config: AppConfig,
}
