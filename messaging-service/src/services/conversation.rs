use crate::models::conversation::Conversation;
use anyhow::{Ok, Result};
use bson::oid::ObjectId;
use bson::Bson;
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::{bson::doc, Client, Collection};

pub struct ConversationService {
    pub collection: Collection<Conversation>,
}

impl ConversationService {
    pub fn new(client: &Client) -> Self {
        let db = client.database("messaging");
        let collection = db.collection("conversations");
        Self { collection }
    }

    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<Conversation>> {
        let filter = doc! { "_id": id };
        let conv = self.collection.find_one(filter).await?;
        Ok(conv)
    }

    pub async fn find_existing(&self, participants: &[String]) -> Result<Option<Conversation>> {
        let filter = doc! {
            "participants": {
                "$all": participants,
                "$size": participants.len() as i32
            }
        };

        let conv = self.collection.find_one(filter).await?;
        Ok(conv)
    }

    pub async fn create(&self, conv: Conversation) -> Result<Conversation> {
        if let Some(existing) = self.find_existing(&conv.participants).await? {
            return Ok(existing);
        }

        let result = self.collection.insert_one(&conv).await?;
        let mut created = conv;
        created.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(created)
    }

    pub async fn find_by_participant(&self, user_id: &str) -> Result<Vec<Conversation>> {
        let filter = doc! { "participants": user_id };
        let mut cursor = self.collection.find(filter).await?;
        let mut conversations = Vec::new();
        while let Some(conv) = cursor.try_next().await? {
            conversations.push(conv);
        }
        Ok(conversations)
    }

    pub async fn delete(&self, id: ObjectId) -> Result<()> {
        let filter = doc! { "_id": id };
        self.collection.delete_one(filter).await?;
        Ok(())
    }

    pub async fn add_participant(&self, id: ObjectId, user_id: String) -> Result<()> {
        let filter = doc! { "_id": id };
        let now = Utc::now();
        let update = doc! {
            "$addToSet": { "participants": user_id },
            "$set": { "updated_at": Bson::DateTime(bson::DateTime::from_millis(now.timestamp_millis())) }
        };
        self.collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn remove_participant(&self, id: ObjectId, user_id: String) -> Result<()> {
        let filter = doc! { "_id": id };
        let now = Utc::now();
        let update = doc! {
            "$pull": { "participants": user_id },
            "$set": { "updated_at": Bson::DateTime(bson::DateTime::from_millis(now.timestamp_millis())) }
        };
        self.collection.update_one(filter, update).await?;
        Ok(())
    }
}
