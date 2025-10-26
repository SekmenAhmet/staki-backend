use crate::models::message::Message;
use anyhow::Result;
use bson::oid::ObjectId;
use futures::stream::TryStreamExt;
use mongodb::{bson::doc, Client, Collection};

pub struct MessageService {
    pub collection: Collection<Message>,
}

impl MessageService {
    pub fn new(client: &Client) -> Self {
        let db = client.database("messaging");
        let collection = db.collection("messages");
        Self { collection }
    }

    pub async fn send_message(&self, msg: Message) -> Result<String> {
        let result = self.collection.insert_one(msg).await?;
        let id = result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get inserted ID"))?
            .to_hex();
        Ok(id)
    }

    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<Message>> {
        let filter = doc! { "_id": id };
        let msg = self.collection.find_one(filter).await?;
        Ok(msg)
    }

    pub async fn update_read_status(&self, id: ObjectId, read: bool) -> Result<()> {
        let filter = doc! { "_id": id };
        let update = doc! { "$set": { "read": read } };
        self.collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn delete_message(&self, id: ObjectId) -> Result<()> {
        let filter = doc! { "_id": id };
        self.collection.delete_one(filter).await?;
        Ok(())
    }

    pub async fn get_messages_by_conversation(
        &self,
        conv_id: ObjectId,
        skip: i64,
        limit: i64,
    ) -> Result<Vec<Message>> {
        let safe_limit = limit.min(100).max(1);
        let filter = doc! {"conversation_id": conv_id};
        let mut cursor = self
            .collection
            .find(filter)
            .sort(doc! {"sent_at": -1})
            .skip(skip as u64)
            .limit(safe_limit)
            .await?;
        let mut messages = Vec::new();
        while let Some(msg) = cursor.try_next().await? {
            messages.push(msg);
        }
        Ok(messages)
    }
}
