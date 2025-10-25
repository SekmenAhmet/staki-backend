use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use shared::validate_token;

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let claims = validate_token(token, &jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
