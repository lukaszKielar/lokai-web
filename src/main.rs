#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
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
    use uuid::Uuid;

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
        .await
        .unwrap();
    }
    {
        let db_pool = db_pool.clone();
        let conversation = lokai::models::Conversation::new(
            Uuid::new_v4(),
            String::from("conversation 2 very loooooooooooong name"),
        );
        let _ = db::create_conversation(db_pool, conversation)
            .await
            .unwrap();
    }

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
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

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
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
        use leptos::logging;
        use uuid::Uuid;
        use lokai::state::AppState;
        use lokai::server::ollama::{OllamaChatResponseStream,OllamaChatParams, default_model};
        use lokai::server::db;
        use futures_util::StreamExt;
        use futures_util::SinkExt as _;

        pub async fn websocket(ws: WebSocketUpgrade, State(app_state): State<AppState>) -> Response {
            logging::log!("ws: {:?}", ws);
            ws.on_upgrade(|socket| handle_socket(socket, app_state))
        }

        async fn handle_socket(socket: WebSocket, app_state: AppState) {
            logging::log!("socket: {:?}", socket);
            let (inference_request_tx, mut inference_request_rx) = tokio::sync::mpsc::channel::<models::Message>(100);
            let (inference_response_tx, mut inference_response_rx) = tokio::sync::mpsc::channel::<models::Message>(100);

            // inference thread
            let inference_thread = tokio::spawn(async move {
                logging::log!("inference thread started");
                while let Some(user_prompt) = inference_request_rx.recv().await {
                    let inference_response_tx_clone = inference_response_tx.clone();
                    let app_state_clone = app_state.clone();
                    inference(user_prompt, inference_response_tx_clone, app_state_clone).await;
                };
                logging::log!("inference thread exited");
            });

            let (mut sender, mut receiver) = socket.split();

            // receiver thread
            let _ = tokio::spawn(async move {
                while let Some(assistant_response_chunk) = inference_response_rx.recv().await {
                    logging::log!("got assistant response chunk: {:?}", assistant_response_chunk);
                    let assistant_response_chunk_json = serde_json::to_string(&assistant_response_chunk).unwrap();
                    if sender.send(WebSocketMessage::Text(assistant_response_chunk_json)).await.is_err() {
                        // client disconnected
                        return ;
                    }
                };
            });

            // sender thread
            let _ = tokio::spawn(async move {
                while let Some(Ok(WebSocketMessage::Text(user_prompt))) = receiver.next().await {
                    logging::log!("message received through the socket: {:?}", user_prompt);
                    let user_prompt = serde_json::from_str::<models::Message>(&user_prompt).unwrap();
                    let _ = inference_request_tx.send(user_prompt).await;
                }
            });

            // https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs
            // tokio::select! {
            //     rv_a = (&mut assistant_response) => {
            //         match rv_a {
            //             Ok(_) => println!("rv_a arm Ok"),
            //             Err(_) => println!("rv_a arm Err"),
            //         }
            //         user_prompt_request.abort();
            //     },
            //     rv_b = (&mut user_prompt_request) => {
            //         match rv_b {
            //             Ok(_) => println!("rv_a arm Ok"),
            //             Err(_) => println!("rv_a arm Err"),
            //         }
            //         assistant_response.abort();
            //     },
            // }

            let _ = inference_thread.await;
            logging::log!("I exited!")
        }

        // TODO: use transactions
        async fn inference(user_prompt: models::Message, inference_response_tx: tokio::sync::mpsc::Sender<models::Message>, app_state: AppState) {
            logging::log!("got user prompt for inference: {:?}", user_prompt);
            let client = app_state.reqwest_client;

            let conversation = models::Conversation::new(user_prompt.conversation_id, user_prompt.content.clone());
            let conversation_id = conversation.id;
            // TODO: create background thread that creates summary of the question,
            // and use it when saving conversation to the DB
            db::create_conversation_if_not_exists(app_state.db_pool.clone(), conversation).await;
            let mut messages = db::get_conversation_messages(app_state.db_pool.clone(), conversation_id).await.unwrap();
            messages.push(user_prompt.clone());

            {
                let db_pool = app_state.db_pool.clone();
                tokio::spawn(async move {
                    let _ = db::create_message(db_pool, user_prompt).await;
                });
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
                .await
                .unwrap()
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
            logging::log!("inference done");
        }
    }
}
