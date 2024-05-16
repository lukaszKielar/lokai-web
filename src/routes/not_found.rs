use axum::extract::State;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::state::SharedAppState;

#[derive(Serialize)]
struct NotFoundTemplate {
    pub message: String,
}

pub async fn not_found(state: State<SharedAppState>) -> impl IntoResponse {
    state.render("not_found.html")
}
