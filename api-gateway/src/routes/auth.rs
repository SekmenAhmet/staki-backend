use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    response::IntoResponse,
};
use reqwest::Client;

use crate::config::Config;

pub async fn proxy_to_auth(
    State(config): State<Arc<Config>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let client = Client::new();
    let uri = req.uri();
    let path = uri.path();

    let url = format!("{}{}", config.auth_service_url, path);

    let method = req.method().clone();
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .unwrap_or_default();

    let response = client
        .request(method, &url)
        .headers(headers)
        .body(body)
        .send()
        .await;

    match response {
        Ok(res) => {
            let status = res.status();
            let headers = res.headers().clone();
            let body = res.bytes().await.unwrap_or_default();

            (status, headers, body)
        }
        Err(_) => (
            axum::http::StatusCode::BAD_GATEWAY,
            axum::http::HeaderMap::new(),
            axum::body::Bytes::new(),
        ),
    }
}
