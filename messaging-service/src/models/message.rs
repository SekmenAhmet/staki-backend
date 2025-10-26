use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<ObjectId>,
    pub conversation_id: ObjectId,
    pub sender_id: String,
    pub content: String,
    pub sent_at: DateTime<Utc>,
    pub read: bool,
}
