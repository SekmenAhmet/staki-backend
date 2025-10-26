use anyhow::{Ok, Result};
use bson::oid::ObjectId;
use mongodb::{Client, Collection, bson::doc};

use crate::models::post::Post;

pub struct PostService {
    pub collection: Collection<Post>,
}

impl PostService {
    pub fn new(client: &Client) -> Self {
        let db = client.database("post");
        let collection = db.collection("post");
        Self { collection }
    }

    pub async fn find_by_id(&self, post_id: ObjectId) -> Result<Option<Post>> {
        let filter = doc! {"_id": post_id};
        let post = self.collection.find_one(filter).await?;
        Ok(post)
    }

    pub async fn create(&self, post: Post) -> Result<Post> {
        let result = self.collection.insert_one(&post).await?;
        let mut created = post;
        created.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(created)
    }

    pub async fn delete(&self, post_id: ObjectId) -> Result<()> {
        let filter = doc! {"_id": post_id};
        self.collection.delete_one(filter).await?;
        Ok(())
    }

    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Post>> {
        let filter = doc! {
            "user_id": user_id,
            "is_deleted": false
        };
        let mut cursor = self.collection.find(filter).await?;
        let mut posts = Vec::new();

        use futures::stream::StreamExt;
        while let Some(result) = cursor.next().await {
            posts.push(result?);
        }

        Ok(posts)
    }
}
