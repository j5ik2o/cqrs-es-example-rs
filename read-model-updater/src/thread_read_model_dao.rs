use anyhow::Result;
use sqlx::MySqlPool;

use cqrs_es_example_domain::thread::events::{ThreadCreated, ThreadDeleted, ThreadMemberAdded, ThreadMemberRemoved, ThreadMessageDeleted, ThreadMessagePosted, ThreadRenamed};

#[async_trait::async_trait]
pub trait ThreadReadModelDao {
    async fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()>;
    async fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()>;
    async fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()>;
    async fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()>;
    async fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()>;
    async fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()>;
    async fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()>;
}

pub struct MockThreadReadModelDao;

#[async_trait::async_trait]
impl ThreadReadModelDao for MockThreadReadModelDao {
    async fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()> {
        Ok(())
    }

    async fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()> {
        Ok(())
    }

    async fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()> {
        Ok(())
    }

    async fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()> {
        Ok(())
    }

    async fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()> {
        Ok(())
    }

    async fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()> {
        Ok(())
    }

    async fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()> {
        Ok(())
    }
}


pub struct ThreadReadModelDaoImpl {
    pool: MySqlPool,
}

impl ThreadReadModelDaoImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ThreadReadModelDao for ThreadReadModelDaoImpl {
    async fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()> {
        let id = thread_created.id.to_string();
        let name = thread_created.name.to_string();
        let administrator_id = thread_created.members.administrator_id().user_account_id.to_string();
        let created_at = thread_created.occurred_at;
        sqlx::query!(
            r#"
            INSERT INTO threads (id, name, owner_id, created_at)
            VALUES (?, ?, ?, ?)
            "#,
            id, name, administrator_id, created_at
        ).execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()> {
        Ok(())
    }

    async fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()> {
        Ok(())
    }

    async fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()> {
        Ok(())
    }

    async fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()> {
        Ok(())
    }

    async fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()> {
        Ok(())
    }

    async fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()> {
        Ok(())
    }
}

