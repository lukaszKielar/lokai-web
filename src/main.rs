use uuid::Uuid;

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
        let conversation =
            lokai::models::Conversation::new(String::from("conversation 2 strasznie dluga nazwa"));
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
        use axum::extract::{ws::{WebSocket, Message as AxumWSMessage, WebSocketUpgrade},  State};
        use axum::response::Response;
        use lokai::models;
        use leptos::logging;
        use lokai::state::AppState;
        use lokai::server::ollama::{OllamaChatResponseStream,OllamaChatParams, default_model};
        use lokai::server::db;
        use futures_util::StreamExt;

        pub async fn websocket(ws: WebSocketUpgrade, State(app_state): State<AppState>) -> Response {
            ws.on_upgrade(|socket| handle_socket(socket, app_state))
        }

        async fn handle_socket(mut socket: WebSocket, app_state: AppState) {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

            while let Some(msg) = socket.recv().await {
                let tx = tx.clone();
                let app_state = app_state.clone();
                logging::log!("socket.recv(): {:?}", msg);
                if let Ok(AxumWSMessage::Text(payload)) = msg {
                    logging::log!("AxumWSMessage::Text: {:?}", payload);
                    let user_prompt = serde_json::from_str::<models::Message>(&payload).unwrap();
                    tokio::spawn(async move {

                        let assistant_message = models::Message::assistant("".to_string(), user_prompt.conversation_id);
                        let _ = db::create_message(app_state.db_pool.clone(), assistant_message.clone());

                        inference(user_prompt, assistant_message.id, tx, app_state).await
                    });
                } else {
                    // client disconnected
                    return;
                };

                while let Some(msg) = rx.recv().await {
                    let msg = AxumWSMessage::Text(msg);
                    if socket.send(msg).await.is_err() {
                        // client disconnected
                        return ;
                    }
                }
            };
        }

        async fn inference(user_prompt: models::Message, assistant_message_id: Uuid, tx: tokio::sync::mpsc::Sender<String>, app_state: AppState) {
            logging::log!("Got user prompt for inference: {:?}", user_prompt);

            let client = app_state.reqwest_client;
            let conversation_id = user_prompt.conversation_id;

            let mut conversation = db::get_conversation_messages(app_state.db_pool.clone(), conversation_id).await.unwrap();
            conversation.push(user_prompt.clone());

            tokio::spawn(async move {
                if db::create_message(app_state.db_pool, user_prompt).await.is_err() {
                    logging::log!("error while saving user message");
                }
            });

            let params = OllamaChatParams {
                model: default_model(),
                messages: conversation.into_iter().map(|m| m.into()).collect(),
                stream: true,
            };

            let mut stream = client
                .post("http://host.docker.internal:11434/api/chat")
                .json(&params)
                .send()
                .await
                .unwrap()
                .bytes_stream().map(|chunk| chunk.unwrap()).map(|chunk| serde_json::from_slice::<OllamaChatResponseStream>(&chunk));

                while let Some(chunk) = stream.next().await {
                    if let Ok(chunk) = chunk {
                        let assistant_response = models::Message {
                                id: assistant_message_id,
                                role: models::Role::Assistant.to_string(),
                                content: chunk.message.content,
                                conversation_id
                        };
                        let assistant_response = serde_json::to_string(&assistant_response).unwrap();

                        if tx.send(assistant_response.to_string()).await.is_err() {
                            break;
                        };

                        if chunk.done {
                            break;
                        }
                    }
                }
        }
    }
}
