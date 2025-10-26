use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use deadpool_redis::{Config as RedisConfig, Runtime};
use mongodb::Client as MongoClient;
use std::sync::Arc;
use tokio;
use tracing_subscriber;

mod config;
mod handlers;
mod models;
mod services;

use config::Config;
use handlers::conversation::{
    add_participant, create_conversation, delete_conversation, get_conversation,
    get_conversations_by_user, remove_participant, ConvAppState,
};
use handlers::messaging::{
    delete_message, get_message, get_messages, mark_as_read, send_message, AppState as MsgAppState,
};
use services::conversation::ConversationService;
use services::messaging::MessageService;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();

    let mongo_client = MongoClient::with_uri_str(&config.mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");

    let redis_pool = RedisConfig::from_url(&config.redis_uri)
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool");

    let message_service = Arc::new(MessageService::new(&mongo_client));
    let conversation_service = Arc::new(ConversationService::new(&mongo_client));

    let msg_state = MsgAppState {
        message_service: message_service.clone(),
        conversation_service: conversation_service.clone(),
        redis_pool: Arc::new(redis_pool),
    };
    let conv_state = ConvAppState {
        conversation_service: conversation_service.clone(),
    };

    let msg_router = Router::new()
        .route("/messages", post(send_message))
        .route("/messages/{message_id}", get(get_message).delete(delete_message))
        .route("/messages/{message_id}/read", patch(mark_as_read))
        .route("/conversations/{conversation_id}/messages", get(get_messages))
        .with_state(msg_state);

    let conv_router = Router::new()
        .route("/conversations", post(create_conversation))
        .route(
            "/conversations/{conversation_id}",
            get(get_conversation).delete(delete_conversation),
        )
        .route("/conversations/{conversation_id}/members", post(add_participant))
        .route(
            "/conversations/{conversation_id}/members/{user_id}",
            delete(remove_participant),
        )
        .route("/users/{user_id}/conversations", get(get_conversations_by_user))
        .with_state(conv_state);

    let app = Router::new().merge(msg_router).merge(conv_router);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("> Messaging service running on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
