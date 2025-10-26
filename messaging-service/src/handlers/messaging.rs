use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use std::sync::Arc;

use deadpool_redis::redis::AsyncCommands;

use crate::{
    models::message::Message, services::conversation::ConversationService,
    services::messaging::MessageService,
};
use shared::jwt::AuthenticatedUser;

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default)]
    pub skip: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Clone)]
pub struct AppState {
    pub message_service: Arc<MessageService>,
    pub conversation_service: Arc<ConversationService>,
    pub redis_pool: Arc<deadpool_redis::Pool>,
}

pub async fn get_messages(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(conversation_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let conv_id = ObjectId::parse_str(&conversation_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid conversation id".to_string(),
        )
    })?;

    let conversation = state
        .conversation_service
        .find_by_id(conv_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;

    if !conversation.participants.contains(&user.sub) {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut con = state.redis_pool.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error (Redis)".to_string(),
        )
    })?;

    let cache_key = format!("messages:{}", conversation_id);
    if let Ok(Some(cached)) = con.get::<String, Option<String>>(cache_key.clone()).await {
        if let Ok(messages) = serde_json::from_str::<Vec<Message>>(&cached) {
            return Ok(Json(messages));
        }
    }

    let messages = state
        .message_service
        .get_messages_by_conversation(conv_id, pagination.skip, pagination.limit)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    let serialized = serde_json::to_string(&messages).unwrap();
    let _: () = con.set_ex(cache_key, serialized, 60).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Redis set error".to_string(),
        )
    })?;

    Ok(Json(messages))
}

pub async fn send_message(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut msg): Json<Message>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    if msg.content.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Message content cannot be empty".to_string()));
    }

    if msg.content.len() > 10000 {
        return Err((StatusCode::BAD_REQUEST, "Message content too long (max 10000 characters)".to_string()));
    }

    msg.sender_id = user.sub.clone();
    msg.sent_at = Utc::now();
    msg.read = false;

    let conv_id = msg.conversation_id;
    let conversation = state
        .conversation_service
        .find_by_id(conv_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;

    if !conversation.participants.contains(&user.sub) {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let inserted_id = state
        .message_service
        .send_message(msg.clone())
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    let mut con = state.redis_pool.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error (Redis)".to_string(),
        )
    })?;
    let cache_key = format!("messages:{}", conv_id);
    let _: () = con.del(cache_key).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Redis delete error".to_string(),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "message_id": inserted_id,
            "sent_at": msg.sent_at
        }))
    ))
}

pub async fn get_message(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(message_id): Path<String>,
) -> Result<Json<Message>, (StatusCode, String)> {
    let msg_id = ObjectId::parse_str(&message_id).map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid message id".to_string())
    })?;

    let message = state
        .message_service
        .find_by_id(msg_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Message not found".to_string()))?;

    let conversation = state
        .conversation_service
        .find_by_id(message.conversation_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;

    if !conversation.participants.contains(&user.sub) {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    Ok(Json(message))
}

pub async fn mark_as_read(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(message_id): Path<String>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let msg_id = ObjectId::parse_str(&message_id).map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid message id".to_string())
    })?;

    let message = state
        .message_service
        .find_by_id(msg_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Message not found".to_string()))?;

    let conversation = state
        .conversation_service
        .find_by_id(message.conversation_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;

    if !conversation.participants.contains(&user.sub) {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    state
        .message_service
        .update_read_status(msg_id, true)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    let mut con = state.redis_pool.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error (Redis)".to_string(),
        )
    })?;
    let cache_key = format!("messages:{}", message.conversation_id);
    let _: () = con.del(cache_key).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Redis delete error".to_string(),
        )
    })?;

    Ok((StatusCode::OK, "Message marked as read".to_string()))
}

pub async fn delete_message(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(message_id): Path<String>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let msg_id = ObjectId::parse_str(&message_id).map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid message id".to_string())
    })?;

    let message = state
        .message_service
        .find_by_id(msg_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Message not found".to_string()))?;

    if message.sender_id != user.sub {
        return Err((StatusCode::FORBIDDEN, "Can only delete your own messages".to_string()));
    }

    state
        .message_service
        .delete_message(msg_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    let mut con = state.redis_pool.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error (Redis)".to_string(),
        )
    })?;
    let cache_key = format!("messages:{}", message.conversation_id);
    let _: () = con.del(cache_key).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Redis delete error".to_string(),
        )
    })?;

    Ok((StatusCode::OK, "Message deleted".to_string()))
}
