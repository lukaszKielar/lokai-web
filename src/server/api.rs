use crate::models::{Conversation, Message};
use leptos::{server, ServerFnError};
use uuid::Uuid;

#[cfg(feature = "ssr")]
async fn slow_down_db() {
    use tokio;
    let _ = tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

#[server(AskAssistant, "/api")]
pub async fn ask_assistant(
    user_message: Message,
    new_conversation: bool,
) -> Result<Message, ServerFnError> {
    use super::{db, ollama::*};
    use leptos::use_context;
    use sqlx::SqlitePool;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let conversation_id = user_message.conversation_id;

    {
        if new_conversation {
            let db_pool = db_pool.clone();
            // TODO: handle error when saving conversation to DB
            let _ = db::create_conversation(
                db_pool,
                Conversation {
                    id: conversation_id,
                    name: user_message.content.clone(),
                },
            )
            .await
            .unwrap();
        }
    }

    {
        let db_pool = db_pool.clone();
        let _ = db::create_message(db_pool, user_message).await;
    }

    let messages = {
        let db_pool = db_pool.clone();
        db::get_conversation_messages(db_pool, conversation_id)
            .await
            // TODO: handle result
            .unwrap()
            .into_iter()
            .map(|m| OllamaMessage::from(m))
            .collect()
    };

    // TODO: handle lack of context
    let client = use_context::<reqwest::Client>().expect("reqwest.Client not found");
    let params = OllamaChatParams {
        model: default_model(),
        messages: messages,
        stream: false,
    };

    // TODO: properly handle errors
    let response: OllamaChatResponse = client
        .post("http://host.docker.internal:11434/api/chat")
        .json(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let assistant_message =
        Message::assistant(response.message.content.trim().to_string(), conversation_id);

    {
        let assistant_message = assistant_message.clone();
        tokio::spawn(db::create_message(db_pool, assistant_message));
    }

    Ok(assistant_message)
}

#[server(GetConversationMessages, "/api")]
pub async fn get_conversation_messages(
    conversation_id: Uuid,
) -> Result<Vec<Message>, ServerFnError> {
    use super::db;
    use leptos::use_context;
    use sqlx::SqlitePool;

    // TODO: remove me
    slow_down_db().await;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let conversation = {
        let db_pool = db_pool.clone();
        db::get_conversation(db_pool, conversation_id)
            .await
            .map_err(|err| ServerFnError::new(err))?
            .ok_or(ServerFnError::new("Conversation doesn't exist".to_string()))?
    };
    let messages = db::get_conversation_messages(db_pool, conversation.id)
        .await
        .unwrap();

    Ok(messages)
}

#[server(GetConversations, "/api")]
pub async fn get_conversations() -> Result<Vec<Conversation>, ServerFnError> {
    use super::db;
    use leptos::use_context;
    use sqlx::SqlitePool;

    // TODO: remove me, this is simulating slow loading
    slow_down_db().await;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let conversations = db::get_conversations(db_pool).await.unwrap();

    Ok(conversations)
}
