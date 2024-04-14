use serde::{Deserialize, Serialize};

use super::message::Message;

// TODO: it should contain: id(uuid), messages (vec<Message>), context (optional vec<i32>)
// Message should contain: id (uuid), persona (enum or string - human/assistant), text (string)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Conversation {
    messages: Vec<Message>,
    pub(crate) context: Option<Vec<i32>>,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            context: None,
        }
    }

    pub fn append_message(&mut self, message: Message) {
        self.messages.push(message)
    }

    pub fn modify_last_msg(&mut self, new_txt: String) {
        if let Some(message) = self.messages.last_mut() {
            message.text = new_txt
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Message> + '_ {
        self.messages.iter()
    }
}
