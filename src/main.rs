#![forbid(unsafe_code)]
mod config;
mod db;
mod error;
mod frontend;
mod models;
mod ollama;
mod state;
mod ws;

use crate::error::Result;
use crate::frontend::handlers;
use crate::state::AppState;
use crate::ws::websocket;

use axum::handler::Handler;
use axum::routing::{delete, get, post};
use axum::Router;
use config::CONFIG;
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("lokai=info")
        .with_target(false)
        .with_level(true)
        .init();

    let state = {
        create_db(&CONFIG.database_url).await;
        let migrator = Migrator::new(std::path::Path::new("./migrations"))
            .await
            .expect("Cannot create database migrator");
        let sqlite: SqlitePool = SqlitePoolOptions::new()
            .connect(&CONFIG.database_url)
            .await
            .expect("Could not make pool");
        migrator
            .run(&sqlite)
            .await
            .expect("Cannot run database migrations");

        AppState {
            sqlite: sqlite,
            reqwest_client: reqwest::Client::new(),
        }
    };

    let api_router = Router::new()
        .route(
            "/conversations/form",
            get(handlers::sidebar_new_conversation_form),
        )
        .route("/conversations", post(handlers::create_conversation))
        .route("/conversations/:id", delete(handlers::delete_conversation));

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/c/:id", get(handlers::conversation))
        .route("/ws", get(websocket))
        .nest("/api", api_router)
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .nest_service(
            "/static",
            ServeDir::new("static")
                .not_found_service(handlers::not_found.with_state(state.clone())),
        )
        .fallback(handlers::not_found)
        .with_state(state);

    let addr = CONFIG.lokai_url();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect(&format!("Cannot bind TcpListener to {:?}", addr));
    info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Cannot start server");

    Ok(())
}

async fn create_db<'a>(db_url: &'a str) {
    if !sqlx::Sqlite::database_exists(db_url)
        .await
        .expect("Cannot check if database exists")
    {
        sqlx::Sqlite::create_database(db_url)
            .await
            .expect("Cannot create database");
    }
}
