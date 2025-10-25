use axum::{http::StatusCode, Extension, Json};
use std::sync::Arc;

use crate::models::{AuthResponse, RegisterRequest};
use crate::services::AuthService;

pub async fn register(
    Extension(auth_service): Extension<Arc<AuthService>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    auth_service
        .register(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}
