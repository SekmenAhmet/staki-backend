use std::sync::Arc;

use axum::{http::StatusCode, Extension, Json};

use crate::models::{AuthResponse, LoginRequest};
use crate::services::AuthService;

pub async fn login(
    Extension(auth_service): Extension<Arc<AuthService>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    auth_service
        .login(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e))
}
