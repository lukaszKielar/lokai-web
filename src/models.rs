use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

// TODO: message should be reactive, saying, whenever it changes, I should update UI
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }

    pub fn user(content: String) -> Self {
        Self::new(Role::User, content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new(Role::Assistant, content)
    }
}

// TODO: it should contain: id(uuid), messages (vec<Message>))
// Message should contain: id (uuid), persona (enum or string - human/assistant), text (string)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Conversation {
    pub messages: Vec<Message>,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn append_message(&mut self, message: Message) {
        self.messages.push(message)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Message> + '_ {
        self.messages.iter()
    }
}
