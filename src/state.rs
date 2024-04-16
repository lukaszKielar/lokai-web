use axum::extract::FromRef;
use leptos::LeptosOptions;
use leptos_router::RouteListing;
use sqlx::SqlitePool;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub db_pool: SqlitePool,
    pub reqwest_client: reqwest::Client,
    pub routes: Vec<RouteListing>,
}
