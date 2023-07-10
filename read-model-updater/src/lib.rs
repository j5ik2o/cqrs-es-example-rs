use anyhow::Result;
use aws_lambda_events::dynamodb;
use lambda_runtime::LambdaEvent;
use serde_dynamo::AttributeValue;
use serde_json::Value;
use sqlx::{MySql, MySqlPool, Pool};

use cqrs_es_example_domain::thread::events::{ThreadCreated, ThreadDeleted, ThreadEvent, ThreadMemberAdded, ThreadMemberRemoved, ThreadMessageDeleted, ThreadMessagePosted, ThreadRenamed};

pub trait ThreadReadModelDao {
    fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()>;
    fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()>;
    fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()>;
    fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()>;
    fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()>;
    fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()>;
    fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()>;
}

pub struct MockThreadReadModelDao {}

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


pub async fn update_read_model<D: ThreadReadModelDao>(thread_read_model_dao: &D, event: LambdaEvent<dynamodb::Event>) {
    event.payload.records.iter().for_each(|record| {
        let attribute_values = record.change.new_image.clone().into_inner();
        let payload_str = match attribute_values.get("payload").unwrap() {
            AttributeValue::S(v) => v.clone(),
            _ => panic!("unexpected type"),
        };
        let type_value_str = get_type_string(&payload_str);
        match type_value_str {
            s if s.starts_with("Thread") => {
                let ev = serde_json::from_str::<ThreadEvent>(&payload_str).unwrap();
                match &ev {
                    ThreadEvent::ThreadCreated(body) =>
                        thread_read_model_dao.insert_thread(body).unwrap(),
                    ThreadEvent::ThreadDeleted(body) =>
                        thread_read_model_dao.delete_thread(body).unwrap(),
                    ThreadEvent::ThreadRenamed(body) =>
                        thread_read_model_dao.update_thread_name(body).unwrap(),
                    ThreadEvent::ThreadMemberAdd(body) =>
                        thread_read_model_dao.insert_member(body).unwrap(),
                    ThreadEvent::ThreadMemberRemoved(body) =>
                        thread_read_model_dao.delete_member(body).unwrap(),
                    ThreadEvent::ThreadMessagePosted(body) =>
                        thread_read_model_dao.post_message(body).unwrap(),
                    ThreadEvent::ThreadMessageDeleted(body) =>
                        thread_read_model_dao.delete_message(body).unwrap(),
                    _ => {}
                }
            }
            _ => {}
        }
    });
}

fn get_type_string(payload_str: &String) -> String {
    let parsed: Value = serde_json::from_str(&payload_str).unwrap();
    let type_value = &parsed["type"];
    let type_value_str = type_value.as_str().unwrap();
    type_value_str.to_string()
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use aws_lambda_events::dynamodb::Event;
    use chrono::{TimeZone, Utc};
    use http::{HeaderMap, HeaderValue};
    use lambda_runtime::Context;
    use serde_json;

    use super::*;

    #[tokio::test]
    async fn example_dynamodb_event() {
        let data = include_bytes!("../fixtures/example-dynamodb-event.json");
        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        // println!("output: {}", output);
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);

        //let event = parsed.records.pop().unwrap();
        //let date = Utc.ymd(2016, 12, 2).and_hms(1, 27, 0);
        //assert_eq!(date, event.change.approximate_creation_date_time);

        let mut headers = HeaderMap::new();
        headers.insert(
            "lambda-runtime-aws-request-id",
            HeaderValue::from_static("my-id"),
        );
        headers.insert(
            "lambda-runtime-deadline-ms",
            HeaderValue::from_static("123"),
        );
        let context = Context::try_from(headers).unwrap();
        let le = LambdaEvent::new(parsed, context);

        update_read_model(MockThreadReadModelDao, le).await.unwrap();
    }
}
