use axum::{routing::any, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod config;
mod middleware;
mod routes;

use config::Config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Arc::new(Config::from_env());

    let auth_routes = Router::new().route("/auth/{*path}", any(routes::proxy_to_auth));

    let app = Router::new()
        .merge(auth_routes)
        .layer(CorsLayer::permissive())
        .with_state(config.clone());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind server");

    tracing::info!("API Gateway listening on {}", addr);

    axum::serve(listener, app).await.expect("Server error");
}
