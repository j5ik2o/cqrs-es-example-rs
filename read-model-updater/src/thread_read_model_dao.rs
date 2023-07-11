use anyhow::Result;
use sqlx::MySqlPool;

use cqrs_es_example_domain::thread::events::{ThreadCreated, ThreadDeleted, ThreadMemberAdded, ThreadMemberRemoved, ThreadMessageDeleted, ThreadMessagePosted, ThreadRenamed};

pub trait ThreadReadModelDao {
    fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()>;
    fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()>;
    fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()>;
    fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()>;
    fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()>;
    fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()>;
    fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()>;
}

#[cfg(test)]
pub struct MockThreadReadModelDao {}

#[cfg(test)]
impl ThreadReadModelDao for MockThreadReadModelDao {
    fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()> {
        Ok(())
    }

    fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()> {
        Ok(())
    }

    fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()> {
        Ok(())
    }

    fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()> {
        Ok(())
    }

    fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()> {
        Ok(())
    }

    fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()> {
        Ok(())
    }

    fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()> {
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

impl ThreadReadModelDao for ThreadReadModelDaoImpl {
    fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()> {
        Ok(())
    }

    fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()> {
        Ok(())
    }

    fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()> {
        Ok(())
    }

    fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()> {
        Ok(())
    }

    fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()> {
        Ok(())
    }

    fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()> {
        Ok(())
    }

    fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()> {
        Ok(())
    }
}

