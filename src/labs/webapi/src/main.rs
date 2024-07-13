use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use opentel::init_trace;
use opentelemetry::global;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::signal;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use webapi_app_state::AppState;

mod opentel;
mod webapi_app_state;
mod flex_db_api;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    // initialize tracing
    let tracer = init_trace("webapi")?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber)?;

    // create a new app state
    let app_state = webapi_app_state::AppState::new().await?;
    // let app_state: AppSharedState = Arc::new(app_state);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /echo` goes to `echo`
        .route("/echo", post(ehco))
        // POST /save goes to `save_to_db`
        .route("/save", post(save_to_db))
        .with_state(app_state.clone())
        // API for flex_db
        .nest("/api", flex_db_api::flex_db_api(app_state))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user));

    // let the world know that server is up
    println!("Server is up and running on port 3086");
    println!("Visit http://localhost:3086");
    println!("Press Ctrl+C to stop the server");

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3086").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // tracer.in_current_span();
    global::shutdown_tracer_provider();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

// basic handler that responds with a static string
#[tracing::instrument]
async fn root() -> &'static str {
    info!("Calling root");
    "Hello, World!"
}

#[tracing::instrument]
async fn ehco(Json(payload): Json<JsonValue>) -> (StatusCode, Json<JsonValue>) {
    (StatusCode::OK, Json(payload))
}

#[tracing::instrument]
async fn save_to_db(
    State(app_state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> (StatusCode, Json<JsonValue>) {
    let _save_result: Vec<JsonValue> = app_state
        .db
        .clone()
        .create("records")
        .content(&payload)
        .await
        .unwrap();
    (StatusCode::OK, Json(payload))
}

#[tracing::instrument]
async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    info!("Creating user with username: {}", payload.username);
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Debug, Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
