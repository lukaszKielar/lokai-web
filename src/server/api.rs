use crate::models::Message;
use leptos::{server, ServerFnError};
use uuid::Uuid;

// TODO: save every prompt, response and context to database, async thread
// TODO: this function should take id of the conversation, prompt and context (history of conversation)
#[server(Chat, "/api", "Url", "chat")]
pub async fn chat(conversation_id: Uuid, user_message: Message) -> Result<Message, ServerFnError> {
    use super::{db, ollama::*};
    use leptos::use_context;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");

    // TODO: that should be different call to server
    let _ = db::save_message(&db_pool, user_message, conversation_id).await;

    let messages = db::get_conversation_messages(&db_pool, conversation_id)
        .await
        // TODO: handle result
        .unwrap()
        .into_iter()
        .map(|m| OllamaMessage::from(m))
        .collect();

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

    let assistant_message = Message::assistant(response.message.content, conversation_id);

    {
        let assistant_message = assistant_message.clone();
        // TODO: I should spawn new thread and return, and then come back and check the result
        let _ = db::save_message(&db_pool, assistant_message, conversation_id).await;
    }

    Ok(assistant_message)
}
