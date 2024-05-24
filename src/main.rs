#![forbid(unsafe_code)]
mod db;
mod error;
mod frontend;
mod models;
mod ollama;
mod state;
mod ws;

use crate::error::Result;
use crate::frontend::handlers::{
    conversation, create_conversation, index, not_found, sidebar_new_conversation_form,
};
use crate::state::AppState;
use crate::ws::websocket;

use axum::handler::Handler;
use axum::routing::{get, post};
use axum::Router;
use lazy_static::lazy_static;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::env;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

lazy_static! {
    pub static ref LOKAI_DEFAULT_LLM_MODEL: String = "phi3:3.8b".to_string();
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("lokai=info")
        .with_target(false)
        .with_level(true)
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let sqlite: SqlitePool = SqlitePoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Could not make pool.");

    let state = AppState {
        sqlite: sqlite,
        reqwest_client: reqwest::Client::new(),
    };

    let api_router = Router::new()
        .route("/conversations/form", get(sidebar_new_conversation_form))
        .route("/conversations", post(create_conversation));

    let app = Router::new()
        .route("/", get(index))
        .route("/c/:id", get(conversation))
        .route("/ws", get(websocket))
        .nest("/api", api_router)
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .nest_service(
            "/static",
            ServeDir::new("static").not_found_service(not_found.with_state(state.clone())),
        )
        .fallback(not_found)
        .with_state(state);

    // TODO: use ENV VAR to define the address
    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect(&format!("Cannot bind TcpListener to {:?}", addr));
    info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Cannot start server");

    Ok(())
}
