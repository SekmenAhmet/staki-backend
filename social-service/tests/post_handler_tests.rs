use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use mongodb::Client as MongoClient;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

use social_service::handlers::post::{
    create_post, delete_post, get_post_by_id, get_user_posts, PostAppState,
};
use social_service::services::post::PostService;

async fn setup_test_app() -> Router {
    // Set JWT_SECRET for tests
    std::env::set_var("JWT_SECRET", "test_secret");

    // Connect to test MongoDB with authentication
    let mongo_client = MongoClient::with_uri_str("mongodb://admin:admin123@localhost:27017/?authSource=admin")
        .await
        .expect("Failed to connect to MongoDB");

    let post_service = Arc::new(PostService::new(&mongo_client));

    let state = PostAppState { post_service };

    Router::new()
        .route("/posts", axum::routing::post(create_post))
        .route(
            "/posts/{post_id}",
            axum::routing::get(get_post_by_id).delete(delete_post),
        )
        .route("/users/{user_id}/posts", axum::routing::get(get_user_posts))
        .with_state(state)
}

fn create_test_token() -> String {
    // Generate a test JWT token with the same secret used in setup_test_app
    shared::generate_token("test_user_123", "test@example.com", "test_secret")
        .expect("Failed to generate token")
}

#[tokio::test]
async fn test_create_post_success() {
    let app = setup_test_app().await;
    let token = create_test_token();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/posts")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "This is a test post"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    if status != StatusCode::CREATED {
        let error_msg = String::from_utf8_lossy(&body);
        panic!("Expected 201 CREATED, got {}. Error: {}", status, error_msg);
    }

    assert_eq!(status, StatusCode::CREATED);

    let post: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(post["content"], "This is a test post");
    assert_eq!(post["user_id"], "test_user_123");
    assert_eq!(post["likes_count"], 0);
    assert_eq!(post["comments_count"], 0);
    assert_eq!(post["is_deleted"], false);
}

#[tokio::test]
async fn test_create_post_without_auth() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/posts")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "This is a test post"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_post_by_id_not_found() {
    let app = setup_test_app().await;
    let token = create_test_token();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/posts/507f1f77bcf86cd799439011")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_post_by_id_invalid_id() {
    let app = setup_test_app().await;
    let token = create_test_token();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/posts/invalid_id")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_post_not_found() {
    let app = setup_test_app().await;
    let token = create_test_token();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/posts/507f1f77bcf86cd799439011")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_user_posts() {
    let app = setup_test_app().await;
    let token = create_test_token();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users/test_user_123/posts")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let posts: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should return an array (verified by the type itself)
}
