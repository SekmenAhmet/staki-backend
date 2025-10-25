use axum::{http::StatusCode, Extension, Json};
use bson::doc;
use mongodb::Database;
use shared::Claims;
use std::sync::Arc;

use crate::models::{User, UserResponse};

pub async fn get_me(
    Extension(claims): Extension<Claims>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<UserResponse>, StatusCode> {
    let users = db.collection::<User>("users");

    let user = users
        .find_one(doc! { "_id": bson::oid::ObjectId::parse_str(&claims.sub).ok() })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse {
        id: claims.sub,
        email: claims.email,
        username: user.username,
    }))
}
