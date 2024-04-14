use serde::{Deserialize, Serialize};

// TODO: message should be reactive, saying, whenever it changes, I should update UI
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub is_human: bool,
    pub text: String,
}

impl Message {
    fn new(is_human: bool, text: String) -> Self {
        Self {
            is_human: is_human,
            text: text,
        }
    }

    pub fn human(text: String) -> Self {
        Self::new(true, text)
    }

    pub fn assistant(text: String) -> Self {
        Self::new(false, text)
    }
}
