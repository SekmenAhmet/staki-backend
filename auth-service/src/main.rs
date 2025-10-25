use axum::{
    routing::{get, post},
    Extension, Router,
};
use mongodb::{options::ClientOptions, Client};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod config;
mod handlers;
mod middleware;
mod models;
mod services;

use config::Config;
use services::AuthService;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();

    let client_options = ClientOptions::parse(&config.mongo_uri)
        .await
        .expect("Failed to parse MongoDB URI");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");
    let db = client.database("staki");

    let auth_service = Arc::new(AuthService::new(&db, config.jwt_secret.clone()));

    let public_routes = Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .layer(Extension(auth_service));

    let protected_routes = Router::new()
        .route("/auth/me", get(handlers::get_me))
        .layer(Extension(Arc::new(db.clone())))
        .layer(axum::middleware::from_fn(middleware::auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind server");

    tracing::info!("Auth service listening on {}", addr);

    axum::serve(listener, app).await.expect("Server error");
}
