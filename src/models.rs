use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

impl From<&str> for Role {
    fn from(value: &str) -> Self {
        match value {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => panic!("Unknown Role!"),
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
        }
    }
}

// TODO: message should be reactive, saying, whenever it changes, I should update UI
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Message {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    // TODO: this should be only for ssr
    pub conversation_id: Uuid,
}

impl Message {
    fn new(role: Role, content: String, conversation_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: role.to_string(),
            content,
            conversation_id,
        }
    }

    pub fn user(content: String, conversation_id: Uuid) -> Self {
        Self::new(Role::User, content, conversation_id)
    }

    pub fn assistant(content: String, conversation_id: Uuid) -> Self {
        Self::new(Role::Assistant, content, conversation_id)
    }
}

// TODO: it should contain: id(uuid), messages (vec<Message>))
// Message should contain: id (uuid), persona (enum or string - human/assistant), text (string)
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Conversation {
    pub id: Uuid,
    // TODO: remove messages, they should be loaded separately by different endpoint
    pub messages: Vec<Message>,
}

impl Conversation {
    // TODO: remove id argument, it can be automatically generated
    pub fn new(id: Uuid) -> Self {
        Self {
            id: id,
            messages: Vec::new(),
        }
    }

    pub fn push_user_message(&mut self, content: String) {
        let message = Message::new(Role::User, content, self.id);
        self.push_message(message);
    }

    pub fn push_assistant_message(&mut self, content: String) {
        let message = Message::new(Role::Assistant, content, self.id);
        self.push_message(message);
    }

    fn push_message(&mut self, message: Message) {
        self.messages.push(message)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Message> + '_ {
        self.messages.iter()
    }
}
