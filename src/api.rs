#[cfg(feature = "ssr")]
use crate::api::{
    db::{get_conversation, save_message},
    ollama::{default_model, OllamaChatParams, OllamaChatResponse, OllamaMessage},
};
use crate::models::{Conversation, Message};
use leptos::server;
use leptos::ServerFnError;
use uuid::Uuid;

// TODO: move to separate module backend/db
#[cfg(feature = "ssr")]
mod db {
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::models::{Conversation, Message};

    // TODO: save conversation if not exist

    pub async fn get_conversation(
        pool: SqlitePool,
        conversation_id: Uuid,
    ) -> Result<Conversation, String> {
        let items: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
    SELECT id, role, content FROM messages
    WHERE conversation_id = ?
            "#,
        )
        .bind(conversation_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        let conversation = Conversation {
            id: conversation_id,
            messages: items
                .iter()
                .map(|(id, role, content)| Message {
                    id: *id,
                    role: role.to_string(),
                    content: content.to_string(),
                    conversation_id: conversation_id,
                })
                .collect(),
        };

        Ok(conversation)
    }

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
#[server(Chat, "/api/chat")]
pub async fn chat(
    // TODO: I need to reduce amount of data send over the wire (maybe conversation_id and fetch it from the DB on a server?)
    conversation_id: Uuid,
    user_message: Message,
) -> Result<Message, ServerFnError> {
    use leptos::{logging, use_context};
    use reqwest;
    use sqlx::SqlitePool;

    logging::log!("got request");

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let db_pool_clone = db_pool.clone();
    let _ = save_message(db_pool_clone, user_message, conversation_id).await;

    let db_pool_clone = db_pool.clone();
    let conversation = get_conversation(db_pool_clone, conversation_id)
        .await
        .unwrap();

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
    let assistant_message = Message::assistant(response.message.content, conversation_id);

    let assistant_message_clone = assistant_message.clone();
    let _ = save_message(db_pool, assistant_message_clone, conversation_id).await;

    Ok(assistant_message)
}
