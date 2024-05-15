use axum::{body::Body, extract::State, http::Request, response::IntoResponse};
use axum_htmx::HxBoosted;
use minijinja::context;
use serde::Serialize;

use crate::state::SharedAppState;

#[derive(Serialize)]
struct NotFoundTemplate {
    pub message: String,
}

pub async fn not_found(
    boosted: HxBoosted,
    state: State<SharedAppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    let message = format!("{:?} not found", req.uri().path());

    state.render_with_context(
        boosted,
        "not_found.html",
        context! {
            message
        },
    )
}
