use axum::extract::FromRef;
use leptos::LeptosOptions;
use leptos_router::RouteListing;

// TODO: add DB pool
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub reqwest_client: reqwest::Client,
    pub routes: Vec<RouteListing>,
}
