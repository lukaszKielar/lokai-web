use sqlx::SqlitePool;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::models::{Conversation, Message};
use crate::server::db;
use crate::server::error::Result;

pub async fn get_conversation_messages(
    db_pool: SqlitePool,
    conversation_id: Uuid,
) -> Result<Vec<Message>> {
    let messages: Vec<Message> = sqlx::query_as(
        r#"
SELECT id, role, content, conversation_id
FROM messages
WHERE conversation_id = ?
        "#,
    )
    .bind(conversation_id)
    .fetch_all(&db_pool)
    .await
    // TODO: handle errors
    .unwrap();

    Ok(messages)
}

// TODO: automatically generate id, I shouldn't create it on a client side
pub async fn create_message(db_pool: SqlitePool, message: Message) -> Result<Message> {
    debug!(message_id = message.id.to_string(), "saving message to db");

    let new_message: Message = sqlx::query_as(
        r#"
INSERT INTO messages ( id, role, content, conversation_id )
VALUES ( ?1, ?2, ?3, ?4 )
RETURNING *
        "#,
    )
    .bind(message.id)
    .bind(message.role)
    .bind(message.content)
    .bind(message.conversation_id)
    .fetch_one(&db_pool)
    .await?;

    debug!(
        message_id = new_message.id.to_string(),
        "message saved to db"
    );

    Ok(new_message)
}

pub async fn get_conversation(
    db_pool: SqlitePool,
    conversation_id: Uuid,
) -> Result<Option<Conversation>> {
    let maybe_conversation: Option<Conversation> = sqlx::query_as(
        r#"
SELECT id, name
FROM conversations
WHERE id = ?
        "#,
    )
    .bind(conversation_id)
    .fetch_optional(&db_pool)
    .await
    // TODO: handle errors
    .unwrap();

    Ok(maybe_conversation)
}

pub async fn get_conversations(db_pool: SqlitePool) -> Result<Vec<Conversation>> {
    let conversations: Vec<Conversation> = sqlx::query_as(
        r#"
SELECT id, name
FROM conversations
        "#,
    )
    .fetch_all(&db_pool)
    .await
    // TODO: handle errors
    .unwrap();

    Ok(conversations)
}

// TODO: automatically generate id, I shouldn't create it on a client side
pub async fn create_conversation(
    db_pool: SqlitePool,
    conversation: Conversation,
) -> Result<Conversation> {
    debug!(
        conversation_id = conversation.id.to_string(),
        "saving conversation to db"
    );

    let new_conversation: Conversation = sqlx::query_as(
        r#"
INSERT INTO conversations ( id, name )
VALUES ( ?1, ?2 )
RETURNING *
        "#,
    )
    .bind(conversation.id)
    .bind(conversation.name)
    .fetch_one(&db_pool)
    .await?;

    debug!(
        conversation_id = new_conversation.id.to_string(),
        "conversation saved to db"
    );

    Ok(new_conversation)
}

pub async fn create_conversation_if_not_exists(db_pool: SqlitePool, conversation: Conversation) {
    match db::get_conversation(db_pool.clone(), conversation.id)
        .await
        .unwrap()
    {
        Some(conversation) => {
            warn!(
                conversation_id = conversation.id.to_string(),
                "conversation already exist in db"
            );
        }
        None => {
            debug!(
                conversation_id = conversation.id.to_string(),
                "conversation doesn't exist in db"
            );
            let _ = db::create_conversation(db_pool, conversation).await;
        }
    };
}

#[cfg(test)]
#[cfg_attr(not(feature = "ssr"), ignore)]
mod tests {
    use crate::models::Role;

    use super::*;
    use sqlx::Row;

    async fn table_count(db_pool: &SqlitePool, table_name: &str) -> Result<i64> {
        let query = format!("SELECT COUNT(*) FROM {table_name}");
        let count = sqlx::query(&query)
            .bind(table_name)
            .fetch_one(db_pool)
            .await?;

        Ok(count.get(0))
    }

    #[sqlx::test]
    async fn test_create_conversation_ok(pool: sqlx::SqlitePool) -> Result<()> {
        // given:
        let pool_clone = pool.clone();
        assert_eq!(table_count(&pool, "conversations").await?, 0);

        // when:
        let new_conversation = create_conversation(
            pool_clone,
            Conversation::new(Uuid::new_v4(), "name".to_string()),
        )
        .await?;

        // then:
        assert_eq!(table_count(&pool, "conversations").await?, 1);
        assert_eq!(new_conversation.name, "name");

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_message_ok(pool: sqlx::SqlitePool) -> Result<()> {
        // given:
        let pool_clone = pool.clone();
        let conversation = create_conversation(
            pool.clone(),
            Conversation::new(Uuid::new_v4(), "name".to_string()),
        )
        .await?;
        assert_eq!(table_count(&pool, "conversations").await?, 1);
        assert_eq!(table_count(&pool, "messages").await?, 0);

        // when:
        let new_message = create_message(
            pool_clone,
            Message::user(Uuid::new_v4(), "content".to_string(), conversation.id),
        )
        .await?;

        // then:
        assert_eq!(table_count(&pool, "messages").await?, 1);
        assert_eq!(new_message.role, Role::User.to_string());
        assert_eq!(new_message.content, "content");
        assert_eq!(new_message.conversation_id, conversation.id);

        Ok(())
    }
}
