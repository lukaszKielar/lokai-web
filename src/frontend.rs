mod templates {
    use askama::Template;

    use crate::models;

    #[derive(Template)]
    #[template(path = "index.html")]
    pub(super) struct Index {
        pub(super) conversations: Vec<models::Conversation>,
    }

    #[derive(Template)]
    #[template(path = "not_found.html")]
    pub(super) struct NotFound;
}

pub mod handlers {
    use super::templates::*;
    use askama_axum::IntoResponse;
    use axum::extract::State;

    use crate::{db, state::AppState};

    pub async fn index(state: State<AppState>) -> impl IntoResponse {
        let conversations = db::get_conversations(state.sqlite.clone()).await.unwrap();
        Index { conversations }
    }

    pub async fn not_found() -> impl IntoResponse {
        NotFound
    }
}
