use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;

use crate::{models::conversation::{Conversation, CreateConversationRequest}, services::conversation::ConversationService};
use shared::jwt::AuthenticatedUser;

#[derive(Clone)]
pub struct ConvAppState {
    pub conversation_service: Arc<ConversationService>,
}

pub async fn create_conversation(
    State(state): State<ConvAppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreateConversationRequest>,
) -> Result<(StatusCode, Json<Conversation>), (StatusCode, String)> {
    let mut participants = req.participants;
    if !participants.contains(&user.sub) {
        participants.push(user.sub.clone());
    }

    let conv = Conversation {
        id: None,
        participants,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created = state.conversation_service.create(conv).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create conversation".to_string(),
        )
    })?;

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get_conversations_by_user(
    State(state): State<ConvAppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Conversation>>, (StatusCode, String)> {
    if user.sub != user_id {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let conversations = state
        .conversation_service
        .find_by_participant(&user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch conversations".to_string(),
            )
        })?;

    Ok(Json(conversations))
}

pub async fn get_conversation(
    State(state): State<ConvAppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(conversation_id): Path<String>,
) -> Result<Json<Conversation>, (StatusCode, String)> {
    let conv_id = mongodb::bson::oid::ObjectId::parse_str(&conversation_id).map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid conversation id".to_string())
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

    Ok(Json(conversation))
}

pub async fn delete_conversation(
    State(state): State<ConvAppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(conversation_id): Path<String>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let conv_id = mongodb::bson::oid::ObjectId::parse_str(&conversation_id).map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid conversation id".to_string())
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

    state
        .conversation_service
        .delete(conv_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete conversation".to_string(),
            )
        })?;

    Ok((StatusCode::OK, "Conversation deleted".to_string()))
}

pub async fn add_participant(
    State(state): State<ConvAppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(conversation_id): Path<String>,
    Json(participant): Json<serde_json::Value>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let conv_id = mongodb::bson::oid::ObjectId::parse_str(&conversation_id).map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid conversation id".to_string())
    })?;

    let user_id = participant["user_id"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "Missing user_id".to_string()))?;

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

    state
        .conversation_service
        .add_participant(conv_id, user_id.to_string())
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to add participant".to_string(),
            )
        })?;

    Ok((StatusCode::OK, "Participant added".to_string()))
}

pub async fn remove_participant(
    State(state): State<ConvAppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((conversation_id, user_id)): Path<(String, String)>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let conv_id = mongodb::bson::oid::ObjectId::parse_str(&conversation_id).map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid conversation id".to_string())
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

    if user_id != user.sub && !conversation.participants.contains(&user.sub) {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    state
        .conversation_service
        .remove_participant(conv_id, user_id.clone())
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to remove participant".to_string(),
            )
        })?;

    Ok((StatusCode::OK, "Participant removed".to_string()))
}
