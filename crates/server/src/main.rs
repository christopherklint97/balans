mod access;
mod assets;
mod auth;
mod config;
mod db;
mod error;
mod filing;
mod k2;
mod models;
mod money;
mod report;
mod routes;
mod sie;
mod tax;
mod validation;

use axum::{middleware, routing::get, Json, Router};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/balans.db".to_string());

    tracing::info!("Connecting to database: {database_url}");

    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Database connected and migrations applied");

    let app_config = config::AppConfig::from_env();
    tracing::info!("App mode: {:?}", app_config.mode);

    // In fixed mode, ensure the company exists (create it on first startup)
    if let (config::AppMode::Fixed, Some(company_id)) =
        (&app_config.mode, &app_config.fixed_company_id)
    {
        let exists =
            sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM companies WHERE id = ?")
                .bind(company_id)
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

        if exists == 0 {
            let now = chrono::Utc::now().to_rfc3339();
            let name = std::env::var("FIXED_COMPANY_NAME")
                .unwrap_or_else(|_| "Mitt Företag".to_string());

            sqlx::query(
                "INSERT INTO companies (id, name, org_number, company_form, fiscal_year_start_month, created_at, updated_at)
                 VALUES (?, ?, '0000000000', 'EF', 1, ?, ?)",
            )
            .bind(company_id)
            .bind(&name)
            .bind(&now)
            .bind(&now)
            .execute(&pool)
            .await
            .expect("Failed to create fixed company");

            db::seed::seed_bas_accounts(&pool, company_id)
                .await
                .expect("Failed to seed BAS accounts for fixed company");

            tracing::info!("Created fixed company '{name}' with id={company_id}");
        }
    }

    let state = config::AppState {
        pool,
        config: app_config,
    };

    // Public routes (no auth required): login, register
    let public_routes = auth::routes::routes();

    // Protected routes (auth required): all business logic
    let protected_routes = routes::api_routes()
        .merge(auth::routes::authenticated_routes())
        .layer(middleware::from_fn(auth::middleware::require_auth));

    let static_dir =
        std::env::var("STATIC_DIR").unwrap_or_else(|_| "frontend/dist".to_string());

    let spa_fallback = ServeDir::new(&static_dir)
        .not_found_service(ServeFile::new(format!("{static_dir}/index.html")));

    let health = Router::new().route("/health", get(|| async {
        Json(serde_json::json!({ "status": "ok" }))
    }));

    let app = Router::new()
        .nest("/api", public_routes.merge(protected_routes).merge(health))
        .fallback_service(spa_fallback)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3100".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {addr}"));

    tracing::info!("Server running on http://localhost:{port}");

    axum::serve(listener, app).await.expect("Server failed");
}
