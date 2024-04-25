use leptos::logging;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{Conversation, Message};

pub async fn get_conversation_messages(
    db_pool: SqlitePool,
    conversation_id: Uuid,
) -> Result<Vec<Message>, String> {
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

// TODO: user proper error handling
pub async fn create_message(db_pool: SqlitePool, message: Message) -> Result<i64, String> {
    logging::log!("saving message to db: {:?}", message.id);

    let id = sqlx::query!(
        r#"
INSERT INTO messages ( id, role, content, conversation_id )
VALUES ( ?1, ?2, ?3, ?4 )
        "#,
        message.id,
        message.role,
        message.content,
        message.conversation_id
    )
    .execute(&db_pool)
    .await
    .unwrap()
    .last_insert_rowid();

    logging::log!("new message saved: {:?}", message.id);

    Ok(id)
}

pub async fn get_conversation(
    db_pool: SqlitePool,
    conversation_id: Uuid,
) -> Result<Option<Conversation>, String> {
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

pub async fn get_conversations(db_pool: SqlitePool) -> Result<Vec<Conversation>, String> {
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

// TODO: user proper error handling
pub async fn create_conversation(
    db_pool: SqlitePool,
    conversation: Conversation,
) -> Result<i64, String> {
    logging::log!("saving conversation to db: {:?}", conversation.id);

    let id = sqlx::query!(
        r#"
INSERT INTO conversations ( id, name )
VALUES ( ?1, ?2 )
        "#,
        conversation.id,
        conversation.name,
    )
    .execute(&db_pool)
    .await
    .unwrap()
    .last_insert_rowid();

    logging::log!("new conversation saved: {:?}", conversation.id);

    Ok(id)
}
