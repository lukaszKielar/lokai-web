use sqlx::SqlitePool;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    models::{Conversation, Message},
    server::db,
};

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
    debug!(message_id = message.id.to_string(), "saving message to db");

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

    debug!(message_id = message.id.to_string(), "message saved to db");

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
    debug!(
        conversation_id = conversation.id.to_string(),
        "saving conversation to db"
    );

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

    debug!(
        conversation_id = conversation.id.to_string(),
        "conversation saved to db"
    );

    Ok(id)
}

// TODO: this doesn't work
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
