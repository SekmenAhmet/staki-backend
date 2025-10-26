use crate::token::validate_token;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
};
use once_cell::sync::Lazy;
use std::env;

static JWT_SECRET: Lazy<String> =
    Lazy::new(|| env::var("JWT_SECRET").expect("JWT_SECRET must be set"));

pub struct AuthenticatedUser(pub crate::token::Claims);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        authenticated_user_from_headers(&parts.headers)
    }
}

pub fn authenticated_user_from_headers(
    headers: &HeaderMap,
) -> Result<AuthenticatedUser, StatusCode> {
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = validate_token(token, &JWT_SECRET).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(AuthenticatedUser(claims))
}
