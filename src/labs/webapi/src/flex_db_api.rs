use axum::{extract::State, http::StatusCode, routing::{get, post, Router}, Json};
use serde_json::{json, Value as JsonValue};

use crate::webapi_app_state::AppState;

pub fn flex_db_api(app_state: AppState) -> Router {
    Router::new()
        // .route("/", get(root))
        .route("/test", get(test))
        .with_state(app_state)
}

async fn test(State(_app_state): State<AppState>) -> (StatusCode, Json<JsonValue>) {
    (StatusCode::OK, Json(json!({"message": "Hello, World!"})))
}
