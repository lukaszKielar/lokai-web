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

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route("/pkg/*path", get(file_and_error_handler))
        .route("/favicon.ico", get(file_and_error_handler))
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
