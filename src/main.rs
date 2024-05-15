#![forbid(unsafe_code)]
mod asset_cache;
mod error;
mod routes;
mod state;

use crate::asset_cache::{AssetCache, SharedAssetCache};
use crate::routes::{route_handler, BaseTemplateData, SharedBaseTemplateData};
use crate::state::{AppState, SharedAppState};

use axum::extract::{Path, Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::{middleware, Router};
use http::header::{CONTENT_ENCODING, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue, StatusCode};
use minijinja::Environment;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::env;

use std::ffi::OsStr;
use tracing::info;

pub type BoxedError = Box<dyn std::error::Error>;

/// Leak a value as a static reference.
pub fn leak_alloc<T>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
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
    let assets: SharedAssetCache = leak_alloc(AssetCache::load_files().await);
    let base_template_data: SharedBaseTemplateData = leak_alloc(BaseTemplateData::new(assets));
    let env = import_templates()?;

    let app_state = leak_alloc(AppState {
        sqlite: sqlite,
        reqwest_client: reqwest::Client::new(),
        assets: assets,
        base_template_data: base_template_data,
        env: env,
    });

    let app = Router::new()
        .merge(route_handler(app_state))
        .nest_service("/assets", static_file_handler(app_state));
    // .route(
    //     "/api/*fn_name",
    //     get(server_fn_handler).post(server_fn_handler),
    // )
    // .route("/pkg/*path", get(file_and_error_handler))
    // .route("/favicon.ico", get(file_and_error_handler))
    // .route("/ws", get(websocket))
    // .route("/*any", get(|| async { Redirect::permanent("/") }))
    // .fallback(file_and_error_handler)
    // .with_state(app_state);

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

fn static_file_handler(state: SharedAppState) -> Router {
    Router::new()
        .route(
            "/:file",
            get(
                |state: State<SharedAppState>, path: Path<String>| async move {
                    let Some(asset) = state.assets.get_from_path(&path) else {
                        return StatusCode::NOT_FOUND.into_response();
                    };

                    let mut headers = HeaderMap::new();

                    // We set the content type explicitly here as it will otherwise
                    // be inferred as an `octet-stream`
                    headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static(asset.ext().unwrap_or("")),
                    );

                    if [Some("css"), Some("js")].contains(&asset.ext()) {
                        headers.insert(CONTENT_ENCODING, HeaderValue::from_static("br"));
                    }

                    // `bytes::Bytes` clones are cheap
                    (headers, asset.contents.clone()).into_response()
                },
            ),
        )
        .layer(middleware::from_fn(cache_control))
        .with_state(state)
}

fn import_templates() -> Result<Environment<'static>, BoxedError> {
    let mut env = Environment::new();

    for entry in std::fs::read_dir("templates")?.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() && path.extension() == Some(OsStr::new("html")) {
            let name = path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or("failed to convert path to string")?
                .to_owned();

            let data = std::fs::read_to_string(&path)?;

            env.add_template_owned(name, data)?;
        }
    }

    Ok(env)
}

async fn cache_control(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
        const CACHEABLE_CONTENT_TYPES: [&str; 6] = [
            "text/css",
            "application/javascript",
            "image/svg+xml",
            "image/webp",
            "font/woff2",
            "image/png",
        ];

        if CACHEABLE_CONTENT_TYPES.iter().any(|&ct| content_type == ct) {
            let value = format!("public, max-age={}", 60 * 60 * 24);

            if let Ok(value) = HeaderValue::from_str(&value) {
                response.headers_mut().insert("cache-control", value);
            }
        }
    }

    response
}

// pub async fn websocket(ws: WebSocketUpgrade, State(app_state): State<AppState>) -> Response {
//     ws.on_upgrade(|socket| handle_socket(socket, app_state))
// }

// async fn handle_socket(socket: WebSocket, app_state: AppState) {
//     debug!("start handling a socket");

//     let (inference_request_tx, mut inference_request_rx) = mpsc::channel::<models::Message>(100);
//     let (inference_response_tx, mut inference_response_rx) = mpsc::channel::<models::Message>(100);
//     let (mut sender, mut receiver) = socket.split();

//     let mut inference_thread = tokio::spawn(async move {
//         info!("inference thread started");
//         while let Some(user_prompt) = inference_request_rx.recv().await {
//             let inference_response_tx_clone = inference_response_tx.clone();
//             let app_state_clone = app_state.clone();
//             match inference(user_prompt, inference_response_tx_clone, app_state_clone).await {
//                 Ok(_) => {}
//                 // consider sending a message through the channel to indicate failure
//                 Err(err) => {
//                     error!(?err, "error while processing inference request, exiting...");
//                     break;
//                 }
//             };
//         }
//         info!("inference thread exited");
//     });

//     let mut sender_thread = tokio::spawn(async move {
//         info!("ws sender thread started");
//         while let Some(assistant_response_chunk) = inference_response_rx.recv().await {
//             debug!(?assistant_response_chunk, "got assistant response chunk");
//             let assistant_response_chunk_json =
//                 match serde_json::to_string(&assistant_response_chunk) {
//                     Ok(value) => value,
//                     Err(err) => {
//                         error!(?err, "cannot serialise assistant response, exiting...");
//                         break;
//                     }
//                 };
//             if sender
//                 .send(WebSocketMessage::Text(assistant_response_chunk_json))
//                 .await
//                 .is_err()
//             {
//                 // client disconnected
//                 return;
//             }
//         }
//         info!("ws sender thread exited");
//     });

//     let mut receiver_thread = tokio::spawn(async move {
//         info!("ws receiver thread started");
//         while let Some(Ok(WebSocketMessage::Text(user_prompt))) = receiver.next().await {
//             debug!(?user_prompt, "user prompt received through websocket");
//             let user_prompt = match serde_json::from_str::<models::Message>(&user_prompt) {
//                 Ok(value) => value,
//                 Err(err) => {
//                     error!(?err, "cannot deserialise user prompt, exiting...");
//                     break;
//                 }
//             };
//             match inference_request_tx.send(user_prompt).await {
//                 Ok(_) => {}
//                 Err(err) => {
//                     error!(?err, "cannot send inference request, exiting...");
//                     break;
//                 }
//             };
//         }
//         info!("ws receiver thread exited");
//     });

//     tokio::select! {
//         inference_thread_result = (&mut inference_thread) => {
//             match inference_thread_result {
//                 Ok(_) => info!("inference thread exited without errors"),
//                 Err(err) => error!(?err, "error returned by inference thread"),
//             }
//             warn!("aborting other threads");
//             sender_thread.abort();
//             receiver_thread.abort();
//         },
//         sender_thread_result = (&mut sender_thread) => {
//             match sender_thread_result {
//                 Ok(_) => info!("sender thread exited without errors"),
//                 Err(err) => error!(?err, "error returned by sender thread"),
//             }
//             warn!("aborting other threads");
//             inference_thread.abort();
//             receiver_thread.abort();
//         },
//         receiver_thread_result = (&mut receiver_thread) => {
//             match receiver_thread_result {
//                 Ok(_) => info!("receiver thread exited without errors"),
//                 Err(err) => error!(?err, "error returned by receiver thread"),
//             }
//             warn!("aborting other threads");
//             sender_thread.abort();
//             inference_thread.abort();
//         },
//     }

//     debug!("finished handling a socket");
// }

// async fn inference(
//     user_prompt: models::Message,
//     inference_response_tx: mpsc::Sender<models::Message>,
//     app_state: AppState,
// ) -> Result<()> {
//     debug!(
//         conversation_id = user_prompt.conversation_id.to_string(),
//         "start inference"
//     );
//     let client = app_state.reqwest_client;

//     let conversation =
//         models::Conversation::new(user_prompt.conversation_id, user_prompt.content.clone());
//     let conversation_id = conversation.id;

//     let _ = db::create_conversation_if_not_exists(app_state.db_pool.clone(), conversation).await;
//     let mut messages =
//         db::get_conversation_messages(app_state.db_pool.clone(), conversation_id).await?;
//     messages.push(user_prompt.clone());

//     {
//         let db_pool = app_state.db_pool.clone();
//         let _ = db::create_message(db_pool, user_prompt).await?;
//     }

//     let params = OllamaChatParams {
//         model: default_model(),
//         messages: messages.into_iter().map(|m| m.into()).collect(),
//         stream: true,
//     };

//     let mut stream = client
//         .post("http://host.docker.internal:11434/api/chat")
//         .json(&params)
//         .send()
//         .await?
//         .bytes_stream()
//         .map(|chunk| chunk.unwrap())
//         .map(|chunk| serde_json::from_slice::<OllamaChatResponseStream>(&chunk));

//     let mut assistant_response =
//         models::Message::assistant(Uuid::new_v4(), "".to_string(), conversation_id);

//     while let Some(chunk) = stream.next().await {
//         if let Ok(chunk) = chunk {
//             assistant_response.update_content(&chunk.message.content);

//             let assistant_response_chunk = models::Message::assistant(
//                 assistant_response.id,
//                 chunk.message.content,
//                 conversation_id,
//             );
//             if inference_response_tx
//                 .send(assistant_response_chunk)
//                 .await
//                 .is_err()
//             {
//                 break;
//             };

//             if chunk.done {
//                 break;
//             }
//         }
//     }

//     let _ = db::create_message(app_state.db_pool, assistant_response).await;
//     debug!(
//         conversation_id = conversation_id.to_string(),
//         "inference done"
//     );

//     Ok(())
// }
