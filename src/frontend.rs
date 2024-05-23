pub(crate) mod templates {
    use askama::Template;

    use crate::models;

    #[derive(Template)]
    #[template(path = "index.html")]
    pub(super) struct Index {
        pub(super) conversations: Vec<models::Conversation>,
    }

    #[derive(Template)]
    #[template(path = "conversation.html")]
    pub(super) struct Conversation {
        pub(super) conversations: Vec<models::Conversation>,
        pub(super) messages: Vec<models::Message>,
    }

    #[derive(Template)]
    #[template(path = "not_found.html")]
    pub(super) struct NotFound;

    #[derive(Template)]
    #[template(path = "chat_area/append_message.html")]
    pub(crate) struct ChatAreaAppendMessage {
        pub message: models::Message,
    }

    #[derive(Template)]
    #[template(path = "chat_area/swap_message.html")]
    pub(crate) struct ChatAreaSwapMessage {
        pub message: models::Message,
    }
}

pub mod handlers {
    use super::templates::*;
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Path, State},
        response::{Redirect, Response},
    };
    use uuid::Uuid;

    use crate::{db, state::AppState};

    pub async fn index(state: State<AppState>) -> impl IntoResponse {
        let conversations = db::get_conversations(state.sqlite.clone()).await.unwrap();
        Index { conversations }
    }

    pub async fn conversation(
        state: State<AppState>,
        Path(conversation_id): Path<String>,
    ) -> Response {
        let conversation_id = match Uuid::parse_str(&conversation_id) {
            Ok(id) => id,
            Err(_) => return Redirect::permanent("/not_found").into_response(),
        };

        let conversations = db::get_conversations(state.sqlite.clone()).await.unwrap();
        if !conversations
            .iter()
            .map(|c| c.id)
            .collect::<Vec<Uuid>>()
            .contains(&conversation_id)
        {
            return Redirect::permanent("/not_found").into_response();
        }

        let messages = db::get_conversation_messages(state.sqlite.clone(), conversation_id)
            .await
            .unwrap();

        Conversation {
            conversations,
            messages,
        }
        .into_response()
    }

    pub async fn not_found() -> impl IntoResponse {
        NotFound
    }
}
