use leptos::{logging, server, use_context, ServerFnError};
use serde::{Deserialize, Serialize};

use crate::models::{Conversation, Message};

// const MODEL: &str = "llama2:7b";
const MODEL: &str = "mistral:7b";
// const MODEL: &str = "tinyllama";

// TODO: I need to save a context of the chat into DB
// that would help when user decided to come back to old conversation
// I won't be feeding model with previous prompts
// asynchronously save everything to DB (maybe in batch mode?? - future consideration)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatResponse {
    pub message: Message,
}

fn default_model() -> String {
    MODEL.to_string()
}

#[derive(Deserialize, Serialize, Debug)]
struct ChatParams {
    #[serde(default = "default_model")]
    model: String,
    messages: Vec<Message>,
    #[serde(default)]
    stream: bool,
}

// TODO: save every prompt, response and context to database, async thread
// TODO: this function should take id of the conversation, prompt and context (history of conversation)
#[server(Chat, "/api/chat")]
pub async fn chat(conversation: Conversation) -> Result<Message, ServerFnError> {
    // TODO: handle lack of context
    let client = use_context::<reqwest::Client>().expect("reqwest.Client not found");
    let params = ChatParams {
        model: default_model(),
        messages: conversation.messages,
        stream: false,
    };
    logging::log!("request params: {:?}", params);

    // TODO: properly handle errors
    let response: ChatResponse = client
        .post("http://host.docker.internal:11434/api/chat")
        .json(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    logging::log!("response: {:?}", response);

    Ok(response.message)
}
