use axum::extract::FromRef;
use sqlx::SqlitePool;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub sqlite: SqlitePool,
    pub reqwest_client: reqwest::Client,
}
