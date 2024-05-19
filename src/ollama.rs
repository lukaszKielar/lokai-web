use serde::{Deserialize, Serialize};

use crate::models::{Message, Role};

pub fn default_model() -> String {
    "phi3:3.8b".to_string()
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OllamaChatResponse {
    pub message: OllamaMessage,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OllamaChatResponseStream {
    pub message: OllamaMessage,
    pub done: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OllamaChatParams {
    #[serde(default = "default_model")]
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    #[serde(default)]
    pub stream: bool,
}
