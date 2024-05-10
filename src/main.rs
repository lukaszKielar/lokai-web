#[cfg(feature = "ssr")]
use lokai::server::error::Result;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<()> {
    use std::env;
    use std::str::FromStr;

    use axum::response::Redirect;
    use axum::routing::get;
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use lokai::app::App;
    use lokai::fileserv::file_and_error_handler;
    use lokai::handlers::{leptos_routes_handler, server_fn_handler};
    use lokai::server::db;
    use lokai::state::AppState;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use tracing::info;
    use uuid::Uuid;

    tracing_subscriber::fmt()
        .with_env_filter("lokai=debug")
        .with_target(false)
        .with_level(true)
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool: SqlitePool = SqlitePoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Could not make pool.");

    // FIXME: hack for now, because I want to understand how resources work in leptos
    let conversation_id = Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap();
    {
        let db_pool = db_pool.clone();
        let _ = sqlx::query(
            r#"
        DELETE FROM messages;
        DELETE FROM conversations;
        INSERT INTO conversations ( id, name )
        VALUES ( ?1, ?2 );
            "#,
        )
        .bind(conversation_id)
        .bind("conversation")
        .execute(&db_pool)
        .await?;
    }
    {
        let db_pool = db_pool.clone();
        let conversation = lokai::models::Conversation::new(
            Uuid::new_v4(),
            String::from("conversation 2 very loooooooooooong name"),
        );
        let _ = db::create_conversation(db_pool, conversation).await?;
    }

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None)
        .await
        .expect("Cannot get LeptosOptions from Cargo.toml");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options,
        db_pool: db_pool,
        reqwest_client: reqwest::Client::new(),
        routes: routes.clone(),
    };

    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route("/pkg/*path", get(file_and_error_handler))
        .route("/favicon.ico", get(file_and_error_handler))
        .route("/ws", get(websocket))
        // TODO: I should add static html for all not found
        .route("/*any", get(|| async { Redirect::permanent("/") }))
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect(&format!("Cannot bind TcpListener to {:?}", addr));
    info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Cannot start server");

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

cfg_if::cfg_if! {
    if #[cfg(feature="ssr")] {
        use axum::extract::{ws::{WebSocket, Message as WebSocketMessage, WebSocketUpgrade},  State};
        use axum::response::Response;
        use lokai::models;
        use uuid::Uuid;
        use tracing::{debug, info, error, warn};
        use lokai::state::AppState;
        use lokai::server::ollama::{OllamaChatResponseStream,OllamaChatParams, default_model};
        use lokai::server::db;
        use futures_util::StreamExt;
        use futures_util::SinkExt as _;
        use tokio::sync::mpsc;

        pub async fn websocket(ws: WebSocketUpgrade, State(app_state): State<AppState>) -> Response {
            ws.on_upgrade(|socket| handle_socket(socket, app_state))
        }

        async fn handle_socket(socket: WebSocket, app_state: AppState) {
            debug!("start handling a socket");

            let (inference_request_tx, mut inference_request_rx) = mpsc::channel::<models::Message>(100);
            let (inference_response_tx, mut inference_response_rx) = mpsc::channel::<models::Message>(100);
            let (mut sender, mut receiver) = socket.split();

            let mut inference_thread = tokio::spawn(async move {
                info!("inference thread started");
                while let Some(user_prompt) = inference_request_rx.recv().await {
                    let inference_response_tx_clone = inference_response_tx.clone();
                    let app_state_clone = app_state.clone();
                    match inference(user_prompt, inference_response_tx_clone, app_state_clone).await {
                        Ok(_) => {},
                        // consider sending a message through the channel to indicate failure
                        Err(err) => {
                            error!(?err, "error while processing inference request, exiting...");
                            break;
                        },
                    };
                };
                info!("inference thread exited");
            });

            let mut sender_thread = tokio::spawn(async move {
                info!("ws sender thread started");
                while let Some(assistant_response_chunk) = inference_response_rx.recv().await {
                    debug!(?assistant_response_chunk, "got assistant response chunk");
                    let assistant_response_chunk_json = match serde_json::to_string(&assistant_response_chunk) {
                        Ok(value) => value,
                        Err(err) => {
                            error!(?err, "cannot serialise assistant response, exiting...");
                            break;
                        },
                    };
                    if sender.send(WebSocketMessage::Text(assistant_response_chunk_json)).await.is_err() {
                        // client disconnected
                        return ;
                    }
                };
                info!("ws sender thread exited");
            });

            let mut receiver_thread = tokio::spawn(async move {
                info!("ws receiver thread started");
                while let Some(Ok(WebSocketMessage::Text(user_prompt))) = receiver.next().await {
                    debug!(?user_prompt, "user prompt received through websocket");
                    let user_prompt = match serde_json::from_str::<models::Message>(&user_prompt) {
                        Ok(value) => value,
                        Err(err) => {
                            error!(?err, "cannot deserialise user prompt, exiting...");
                            break;
                        },
                    };
                    match inference_request_tx.send(user_prompt).await {
                        Ok(_) => {},
                        Err(err) => {
                            error!(?err, "cannot send inference request, exiting...");
                            break;
                        },
                    };
                }
                info!("ws receiver thread exited");
            });

            tokio::select! {
                inference_thread_result = (&mut inference_thread) => {
                    match inference_thread_result {
                        Ok(_) => info!("inference thread exited without errors"),
                        Err(err) => error!(?err, "error returned by inference thread"),
                    }
                    warn!("aborting other threads");
                    sender_thread.abort();
                    receiver_thread.abort();
                },
                sender_thread_result = (&mut sender_thread) => {
                    match sender_thread_result {
                        Ok(_) => info!("sender thread exited without errors"),
                        Err(err) => error!(?err, "error returned by sender thread"),
                    }
                    warn!("aborting other threads");
                    inference_thread.abort();
                    receiver_thread.abort();
                },
                receiver_thread_result = (&mut receiver_thread) => {
                    match receiver_thread_result {
                        Ok(_) => info!("receiver thread exited without errors"),
                        Err(err) => error!(?err, "error returned by receiver thread"),
                    }
                    warn!("aborting other threads");
                    sender_thread.abort();
                    inference_thread.abort();
                },
            }

            debug!("finished handling a socket");
        }

        // TODO: use transactions
        async fn inference(user_prompt: models::Message, inference_response_tx: mpsc::Sender<models::Message>, app_state: AppState) -> Result<()> {
            debug!(conversation_id=user_prompt.conversation_id.to_string(), "start inference");
            let client = app_state.reqwest_client;

            let conversation = models::Conversation::new(user_prompt.conversation_id, user_prompt.content.clone());
            let conversation_id = conversation.id;
            // TODO: create background thread that creates summary of the question,
            // and use it when saving conversation to the DB
            // or simply ask user to put name and pass it in a request
            let _ = db::create_conversation_if_not_exists(app_state.db_pool.clone(), conversation).await;
            let mut messages = db::get_conversation_messages(app_state.db_pool.clone(), conversation_id).await?;
            messages.push(user_prompt.clone());

            {
                let db_pool = app_state.db_pool.clone();
                let _ = db::create_message(db_pool, user_prompt).await?;
            }

            let params = OllamaChatParams {
                model: default_model(),
                messages: messages.into_iter().map(|m| m.into()).collect(),
                stream: true,
            };

            let mut stream = client
                .post("http://host.docker.internal:11434/api/chat")
                .json(&params)
                .send()
                .await?
                .bytes_stream().map(|chunk| chunk.unwrap()).map(|chunk| serde_json::from_slice::<OllamaChatResponseStream>(&chunk));

            let mut assistant_response = models::Message::assistant(Uuid::new_v4(), "".to_string(), conversation_id);

            while let Some(chunk) = stream.next().await {
                if let Ok(chunk) = chunk {
                    assistant_response.update_content(&chunk.message.content);

                    let assistant_response_chunk = models::Message::assistant(assistant_response.id, chunk.message.content, conversation_id);
                    if inference_response_tx.send(assistant_response_chunk).await.is_err() {
                        break;
                    };

                    if chunk.done {
                        break;
                    }
                }
            }

            let _ = db::create_message(app_state.db_pool, assistant_response).await;
            debug!(conversation_id=conversation_id.to_string(), "inference done");

            Ok(())
        }
    }
}
