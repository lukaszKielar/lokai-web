// TODO: handle errors by implementing intoresponse for my db error
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

    #[derive(Template)]
    #[template(path = "sidebar/new_conversation_form.html")]
    pub(crate) struct SidebarNewConversationForm;

    #[derive(Template)]
    #[template(path = "sidebar/conversation.html")]
    pub(crate) struct SidebarConversation {
        pub conversation: models::Conversation,
    }
}

pub mod handlers {
    use super::templates::*;
    use askama_axum::IntoResponse;
    use axum::{
        body::Body,
        extract::{Path, State},
        response::{Redirect, Response},
        Form,
    };
    use http::{HeaderMap, HeaderValue, StatusCode};
    use serde::Deserialize;
    use sqlx::SqlitePool;
    use tracing::error;
    use uuid::Uuid;

    use crate::{db, models, state::AppState};

    pub async fn index(state: State<AppState>) -> impl IntoResponse {
        let conversations = db::get_conversations(state.sqlite.clone()).await.unwrap();
        Index { conversations }
    }

    pub async fn conversation(
        State(sqlite): State<SqlitePool>,
        Path(conversation_id): Path<String>,
    ) -> Response {
        let conversation_id = match Uuid::parse_str(&conversation_id) {
            Ok(id) => id,
            Err(_) => return Redirect::permanent("/not_found").into_response(),
        };

        let conversations = db::get_conversations(sqlite.clone()).await.unwrap();
        if !conversations
            .iter()
            .map(|c| c.id)
            .collect::<Vec<Uuid>>()
            .contains(&conversation_id)
        {
            return Redirect::permanent("/not_found").into_response();
        }

        let messages = db::get_conversation_messages(sqlite, conversation_id)
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

    pub async fn sidebar_new_conversation_form() -> impl IntoResponse {
        SidebarNewConversationForm
    }

    // TODO: add validation, e.g. cannot be empty string
    #[derive(Deserialize, Debug)]
    pub struct NewConversationForm {
        pub conversation_name: String,
    }

    pub async fn create_conversation(
        State(sqlite): State<SqlitePool>,
        Form(new_conversation_form): Form<NewConversationForm>,
    ) -> impl IntoResponse {
        let new_conversation = models::Conversation::new(new_conversation_form.conversation_name);
        // TODO: implement into response for my error
        let new_conversation = db::create_conversation(sqlite, new_conversation)
            .await
            .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "HX-Redirect",
            HeaderValue::from_str(&format!("/c/{:?}", &new_conversation.id)).unwrap(),
        );

        (
            headers,
            SidebarConversation {
                conversation: new_conversation,
            },
        )
    }

    pub async fn delete_conversation(
        State(sqlite): State<SqlitePool>,
        Path(conversation_id): Path<Uuid>,
    ) -> Response {
        match db::delete_conversation(sqlite, conversation_id).await {
            Ok(_) => Body::empty().into_response(),
            Err(err) => {
                error!(
                    conversation_id = conversation_id.to_string(),
                    "Error when deleting conversation: {:?}", err
                );
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
