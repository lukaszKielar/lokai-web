#[cfg(feature = "ssr")]
use crate::api::{
    db::save_message,
    ollama::{default_model, OllamaChatParams, OllamaChatResponse, OllamaMessage},
};
use crate::models::{Conversation, Message};
use leptos::server;
use leptos::ServerFnError;

// TODO: move to separate module backend/db
#[cfg(feature = "ssr")]
mod db {
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::models::Message;

    // TODO: user proper error handling
    pub async fn save_message(
        pool: SqlitePool,
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
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        Ok(id)
    }
}

// TODO: move to separate module backend/ollama
#[cfg(feature = "ssr")]
mod ollama {
    use serde::{Deserialize, Serialize};

    use crate::models::{Message, Role};

    const MODEL: &str = "mistral:7b";
    // const MODEL: &str = "llama2:7b";
    // const MODEL: &str = "tinyllama";

    pub fn default_model() -> String {
        MODEL.to_string()
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct OllamaMessage {
        pub role: Role,
        pub content: String,
    }

    impl From<&Message> for OllamaMessage {
        fn from(value: &Message) -> Self {
            Self {
                role: Role::from(value.role.as_ref()),
                content: value.content.clone(),
            }
        }
    }

    // TODO: I need to save a context of the chat into DB
    // that would help when user decided to come back to old conversation
    // I won't be feeding model with previous prompts
    // asynchronously save everything to DB (maybe in batch mode?? - future consideration)
    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct OllamaChatResponse {
        pub message: OllamaMessage,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct OllamaChatParams {
        #[serde(default = "default_model")]
        pub model: String,
        pub messages: Vec<OllamaMessage>,
        #[serde(default)]
        pub stream: bool,
    }
}

// TODO: save every prompt, response and context to database, async thread
// TODO: this function should take id of the conversation, prompt and context (history of conversation)
#[server(Chat, "/api", "Url", "chat")]
pub async fn chat(
    // TODO: I need to reduce amount of data send over the wire (maybe conversation_id and fetch it from the DB on a server?)
    conversation: Conversation,
    user_message: Message,
) -> Result<Message, ServerFnError> {
    use leptos::{logging, use_context};
    use reqwest;
    use sqlx::SqlitePool;
    use tokio::spawn;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let save_user_message = {
        let db_pool = db_pool.clone();
        let user_message = user_message.clone();
        spawn(async move { save_message(db_pool, user_message, conversation.id) })
    };

    // TODO: handle lack of context
    let client = use_context::<reqwest::Client>().expect("reqwest.Client not found");
    let params = OllamaChatParams {
        model: default_model(),
        messages: conversation
            .iter()
            .map(|m| OllamaMessage::from(m))
            .collect(),
        stream: false,
    };
    logging::log!("request params: {:?}", params);

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

    logging::log!("response: {:?}", response);
    let assistant_message = Message::assistant(response.message.content, conversation.id);
    let save_assistant_message = {
        let db_pool = db_pool.clone();
        let assistant_message = assistant_message.clone();
        spawn(async move { save_message(db_pool, assistant_message, conversation.id) })
    };

    // TODO: handle failures
    let _ = save_user_message.await?;
    let _ = save_assistant_message.await?;

    Ok(assistant_message)
}
