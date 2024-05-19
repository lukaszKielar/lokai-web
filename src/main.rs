#![forbid(unsafe_code)]
mod db;
mod error;
mod frontend;
mod models;
mod ollama;
mod state;
mod ws;

use crate::error::Result;
use crate::frontend::handlers::{conversation, index, not_found};
use crate::state::AppState;
use crate::ws::websocket;

use axum::handler::Handler;
use axum::routing::get;
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::env;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("lokai=debug")
        .with_target(false)
        .with_level(true)
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let sqlite: SqlitePool = SqlitePoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Could not make pool.");

    // create_data(sqlite.clone()).await;

    let state = AppState {
        sqlite: sqlite,
        reqwest_client: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/c/:id", get(conversation))
        .route("/ws", get(websocket))
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

async fn create_data(sqlite: SqlitePool) {
    {
        let sqlite = sqlite.clone();
        let conversation = models::Conversation::new("conversation 1".to_string());
        db::create_conversation(sqlite.clone(), conversation.clone())
            .await
            .unwrap();
        db::create_message(
            sqlite.clone(),
            models::Message::user("why is the sky blue?".to_string(), conversation.id.clone()),
        )
        .await
        .unwrap();
        db::create_message(
            sqlite.clone(),
            models::Message::assistant(
                "it's not blue, it's red!!".to_string(),
                conversation.id.clone(),
            ),
        )
        .await
        .unwrap();
    }
    for i in 2..=20 {
        let sqlite = sqlite.clone();
        let conversation = models::Conversation::new(format!("conversation {i}"));
        db::create_conversation(sqlite, conversation).await.unwrap();
    }
}
