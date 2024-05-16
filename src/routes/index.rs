use axum::{extract::State, response::IntoResponse};
use serde::Serialize;

use crate::state::SharedAppState;

#[derive(Serialize)]
struct IndexTemplate;

pub async fn index(state: State<SharedAppState>) -> impl IntoResponse {
    state.render("index.html")
}
