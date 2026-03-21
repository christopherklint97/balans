pub mod accounts;
pub mod admin;
pub mod annual_report;
pub mod assets;
pub mod closing;
pub mod companies;
pub mod compliance;
pub mod filing;
pub mod fiscal_years;
pub mod reports;
pub mod sie;
pub mod tax;
pub mod vouchers;

use axum::Router;
use crate::config::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(companies::routes())
        .merge(fiscal_years::routes())
        .merge(accounts::routes())
        .merge(vouchers::routes())
        .merge(reports::routes())
        .merge(sie::routes())
        .merge(closing::routes())
        .merge(annual_report::routes())
        .merge(compliance::routes())
        .merge(filing::routes())
        .merge(tax::routes())
        .merge(assets::routes())
        .merge(admin::routes())
}
