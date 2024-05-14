use axum::extract::FromRef;
use sqlx::SqlitePool;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub reqwest_client: reqwest::Client,
}
