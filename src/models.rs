use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use url::Url;
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

#[derive(FromRow, Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Message {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Message {
    fn new(role: Role, content: String, conversation_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            role: role.to_string(),
            content,
            conversation_id,
            created_at: Utc::now(),
        }
    }

    pub fn user(content: String, conversation_id: Uuid) -> Self {
        Self::new(Role::User, content, conversation_id)
    }

    pub fn assistant(content: String, conversation_id: Uuid) -> Self {
        Self::new(Role::Assistant, content, conversation_id)
    }

    pub fn update_content(&mut self, update: &str) {
        self.content.push_str(update);
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct UserPromptFormMessage {
    pub user_prompt: String,
    pub HEADERS: HashMap<String, serde_json::Value>,
}

impl UserPromptFormMessage {
    fn hx_current_url(&self) -> &str {
        // SAFETY: it's safe to unwrap because:
        // - we know we'll get HX-Current-URL header value
        // - we know that HX-Current-URL value is String(String), so as_str returns Some(&str)
        self.HEADERS
            .get("HX-Current-URL")
            .unwrap()
            .as_str()
            .unwrap()
    }

    pub fn conversation_id(&self) -> Uuid {
        // SAFETY: We can unwrap as we know the value set by HTMX is correct
        let url = Url::parse(self.hx_current_url()).unwrap();
        let path = url.path().strip_prefix("/c/").unwrap();
        // SAFETY: We can unwrap because router doesn't allow invalid UUIDs
        Uuid::from_str(path).unwrap()
    }
}

impl From<UserPromptFormMessage> for Message {
    fn from(value: UserPromptFormMessage) -> Self {
        Message::user(value.user_prompt.clone(), value.conversation_id())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, FromRow, PartialEq)]
pub struct Conversation {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Conversation {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_get_conversation_id() {
        // given:
        let mut htmx_headers: HashMap<String, serde_json::Value> = HashMap::new();
        htmx_headers.insert(
            "HX-Current-URL".to_string(),
            json!("http://localhost:3000/c/a310afea-981e-4054-924a-37090ac227e2"),
        );
        let user_prompt_form_message = UserPromptFormMessage {
            user_prompt: "".to_string(),
            HEADERS: htmx_headers,
        };

        // when:
        let conversation_id = user_prompt_form_message.conversation_id();

        // then:
        assert_eq!(
            conversation_id,
            Uuid::from_str("a310afea-981e-4054-924a-37090ac227e2").unwrap()
        );
    }
}
