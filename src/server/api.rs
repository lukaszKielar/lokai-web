use crate::models::{Conversation, Message};
use leptos::{server, ServerFnError};
use uuid::Uuid;

#[server(GetConversationMessages, "/api")]
pub async fn get_conversation_messages(
    conversation_id: Uuid,
) -> Result<Vec<Message>, ServerFnError> {
    use super::db;
    use leptos::use_context;
    use sqlx::SqlitePool;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let conversation = {
        let db_pool = db_pool.clone();
        db::get_conversation(db_pool, conversation_id)
            .await
            .map_err(|err| Into::<ServerFnError>::into(err))?
            .ok_or(ServerFnError::new("Conversation doesn't exist".to_string()))?
    };

    let messages = db::get_conversation_messages(db_pool, conversation.id)
        .await
        .map_err(|err| Into::<ServerFnError>::into(err))?;

    Ok(messages)
}

#[server(GetConversations, "/api")]
pub async fn get_conversations() -> Result<Vec<Conversation>, ServerFnError> {
    use super::db;
    use leptos::use_context;
    use sqlx::SqlitePool;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let conversations = db::get_conversations(db_pool)
        .await
        .map_err(|err| Into::<ServerFnError>::into(err))?;

    Ok(conversations)
}
