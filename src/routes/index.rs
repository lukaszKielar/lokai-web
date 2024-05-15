use axum::{extract::State, response::IntoResponse};
use axum_htmx::HxBoosted;
use serde::Serialize;

use super::BaseTemplateData;
use crate::state::SharedAppState;

#[derive(Serialize)]
struct IndexTemplate {
    base: Option<BaseTemplateData>,
}

pub async fn index(boosted: HxBoosted, state: State<SharedAppState>) -> impl IntoResponse {
    state.render(boosted, "index.html")
}
