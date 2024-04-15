use axum::body::Body as AxumBody;
use axum::extract::State;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use leptos::provide_context;
use leptos_axum::handle_server_fns_with_context;

use crate::app::App;
use crate::state::AppState;

pub async fn server_fn_handler(
    State(app_state): State<AppState>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(app_state.pool.clone());
            provide_context(app_state.reqwest_client.clone());
        },
        request,
    )
    .await
}

pub async fn leptos_routes_handler(
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(app_state.pool.clone());
            provide_context(app_state.reqwest_client.clone());
        },
        App,
    );
    handler(req).await.into_response()
}
