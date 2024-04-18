use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::Message;

pub async fn get_conversation_messages(
    db_pool: &SqlitePool,
    conversation_id: Uuid,
) -> Result<Vec<Message>, String> {
    let rows: Vec<(Uuid, String, String)> = sqlx::query_as(
        r#"
SELECT id, role, content
FROM messages
WHERE conversation_id = ?
        "#,
    )
    .bind(conversation_id)
    .fetch_all(db_pool)
    .await
    // TODO: handle errors
    .unwrap();

    let messages = rows
        .into_iter()
        .map(|(id, role, content)| Message {
            id,
            role,
            content,
            conversation_id,
        })
        .collect();

    Ok(messages)
}

// TODO: user proper error handling
// TODO: rename to create_message
pub async fn save_message(
    db_pool: &SqlitePool,
    message: Message,
    conversation_id: Uuid,
) -> Result<i64, String> {
    let id = sqlx::query!(
        r#"
INSERT INTO messages ( id, role, content, conversation_id )
VALUES ( ?1, ?2, ?3, ?4 )
        "#,
        message.id,
        message.role,
        message.content,
        conversation_id
    )
    .execute(db_pool)
    .await
    .unwrap()
    .last_insert_rowid();

    Ok(id)
}
