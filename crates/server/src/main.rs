mod assets;
mod auth;
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

use axum::{middleware, Router};
use tower_http::cors::CorsLayer;

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

    // Public routes (no auth required): login, register
    let public_routes = auth::routes::routes();

    // Protected routes (auth required): all business logic
    let protected_routes = routes::api_routes()
        .merge(auth::routes::authenticated_routes())
        .layer(middleware::from_fn(auth::middleware::require_auth));

    let app = Router::new()
        .nest("/api", public_routes.merge(protected_routes))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3100".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {addr}"));

    tracing::info!("Server running on http://localhost:{port}");

    axum::serve(listener, app).await.expect("Server failed");
}
