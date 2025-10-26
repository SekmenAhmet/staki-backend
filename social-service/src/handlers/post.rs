use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use mongodb::bson::oid::ObjectId;
use shared::jwt::AuthenticatedUser;

use crate::{
    models::post::{CreatePostRequest, Post},
    services::post::PostService,
};

#[derive(Clone)]
pub struct PostAppState {
    pub post_service: Arc<PostService>,
}

pub async fn create_post(
    State(state): State<PostAppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<Post>), (StatusCode, String)> {
    let post = Post {
        id: None,
        user_id: user.sub.clone(),
        content: req.content,
        likes_count: 0,
        comments_count: 0,
        replies_count: 0,
        is_deleted: false,
        created_at: chrono::Utc::now(),
    };

    match state.post_service.create(post).await {
        Ok(created_post) => Ok((StatusCode::CREATED, Json(created_post))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create post".to_string(),
        )),
    }
}

pub async fn get_post_by_id(
    State(state): State<PostAppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(post_id): Path<String>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let object_id = ObjectId::parse_str(&post_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid post ID format".to_string(),
        )
    })?;

    match state.post_service.find_by_id(object_id).await {
        Ok(Some(post)) => {
            if post.is_deleted {
                Err((StatusCode::NOT_FOUND, "Post not found".to_string()))
            } else {
                Ok(Json(post))
            }
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, "Post not found".to_string())),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch post".to_string(),
        )),
    }
}

pub async fn delete_post(
    State(state): State<PostAppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(post_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let object_id = ObjectId::parse_str(&post_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid post ID format".to_string(),
        )
    })?;

    // Check if post exists and belongs to user
    match state.post_service.find_by_id(object_id).await {
        Ok(Some(post)) => {
            if post.user_id != user.sub {
                return Err((
                    StatusCode::FORBIDDEN,
                    "You can only delete your own posts".to_string(),
                ));
            }
            if post.is_deleted {
                return Err((StatusCode::NOT_FOUND, "Post not found".to_string()));
            }
        }
        Ok(None) => return Err((StatusCode::NOT_FOUND, "Post not found".to_string())),
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete post".to_string(),
            ));
        }
    }

    match state.post_service.delete(object_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete post".to_string(),
        )),
    }
}

pub async fn get_user_posts(
    State(state): State<PostAppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    match state.post_service.find_by_user_id(&user_id).await {
        Ok(posts) => Ok(Json(posts)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch posts".to_string(),
        )),
    }
}
