use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
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
        let value = value.to_lowercase();
        match value.as_ref() {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => panic!("Unknown Role!"),
        }
    }
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        let value = value.to_lowercase();
        match value.as_ref() {
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
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Conversation {
    pub id: Uuid,
    pub name: String,
}

impl Conversation {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
        }
    }
}
