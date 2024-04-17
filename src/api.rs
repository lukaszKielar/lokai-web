#[cfg(feature = "ssr")]
use crate::api::ollama::{default_model, OllamaChatParams, OllamaChatResponse, OllamaMessage};
use crate::models::{Conversation, Message};
use leptos::server;
use leptos::use_context;
use leptos::ServerFnError;
use uuid::Uuid;

// TODO: move to separate module backend/db
#[cfg(feature = "ssr")]
mod db {
    use leptos::logging;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::models::{Conversation, Message};

    pub async fn save_conversation(
        pool: SqlitePool,
        conversation: Conversation,
    ) -> Result<Conversation, String> {
        let _id = sqlx::query!(
            r#"
    INSERT INTO conversations ( id, name )
    VALUES ( ?1, ?2 )
            "#,
            conversation.id,
            "TODO"
        )
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        Ok(conversation)
    }

    pub async fn get_conversation(
        pool: SqlitePool,
        conversation_id: Uuid,
    ) -> Result<Conversation, String> {
        logging::log!("fetching record: {:?}", conversation_id);
        let _conversation_raw = sqlx::query(
            r#"
    SELECT id, name
    FROM conversations
    WHERE id = ?
        "#,
        )
        .bind(conversation_id)
        .fetch_one(&pool)
        .await
        // TODO: handle result
        .unwrap();

        let messages = get_conversation_messages(pool, conversation_id)
            .await
            .unwrap();

        Ok(Conversation {
            id: conversation_id,
            messages: messages,
        })
    }

    pub async fn get_conversation_messages(
        pool: SqlitePool,
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
        .fetch_all(&pool)
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

    impl From<Message> for OllamaMessage {
        fn from(value: Message) -> Self {
            Self {
                role: Role::from(value.role.as_ref()),
                content: value.content,
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
    conversation_id: Uuid,
    user_message: Message,
) -> Result<Message, ServerFnError> {
    use db::{get_conversation_messages, save_message};
    use leptos::logging;
    use reqwest;
    use sqlx::SqlitePool;

    logging::log!("got request");

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let db_pool_clone = db_pool.clone();
    // TODO: that should be different call to server
    let _ = save_message(db_pool_clone, user_message, conversation_id).await;

    let db_pool_clone = db_pool.clone();
    let messages = get_conversation_messages(db_pool_clone, conversation_id)
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
    // TODO: I should spawn new thread and return, and then come back and check the result
    let _ = save_message(db_pool, assistant_message_clone, conversation_id).await;

    Ok(assistant_message)
}

#[server(GetConversation, "/api", "GetJson", "conversations")]
pub async fn get_conversation(conversation_id: Uuid) -> Result<Conversation, ServerFnError> {
    use db::get_conversation;
    use sqlx::SqlitePool;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");

    let conversation = get_conversation(db_pool, conversation_id).await.unwrap();

    Ok(conversation)
}

#[server(CreateConversation, "/api", "Url", "conversations")]
pub async fn create_conversation() -> Result<Conversation, ServerFnError> {
    use db::save_conversation;
    use sqlx::SqlitePool;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");

    let conversation = Conversation::new(Uuid::new_v4());
    save_conversation(db_pool, conversation.clone())
        .await
        .unwrap();

    Ok(conversation)
}
