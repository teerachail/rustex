use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put, Router},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::webapi_app_state::AppState;

pub fn flex_db_api(app_state: AppState) -> Router {
    Router::new()
        // .route("/", get(root))
        .route("/test", get(test))
        // POST /api/:collection goes to `create_entity`
        .route("/:collection", post(create_entity))
        // GET /api/:collection goes to `list_entities`
        .route("/:collection", get(list_entities))
        // GET /api/:collection/:id goes to `get_entity`
        .route("/:collection/:id", get(get_entity))
        // PUT /api/:collection/:id goes to `update_entity`
        .route("/:collection/:id", put(update_entity))
        .with_state(app_state)
}

async fn test(State(_app_state): State<AppState>) -> (StatusCode, Json<JsonValue>) {
    (StatusCode::OK, Json(json!({"message": "Hello, World!"})))
}

async fn create_entity(
    State(_app_state): State<AppState>,
    Path(collection): Path<String>,
    Json(payload): Json<JsonValue>,
) -> (StatusCode, Json<JsonValue>) {
    let db = _app_state.db.clone();
    let result: Vec<JsonValue> = db.create(collection).content(payload).await.unwrap();
    let id = result.first().unwrap().get("id").unwrap();
    let id = id.get("id").unwrap().get("String").unwrap().as_str().unwrap();
    (StatusCode::OK, Json(json!({"id": id})))
}

async fn list_entities(
    State(_app_state): State<AppState>,
    Path(collection): Path<String>,
) -> (StatusCode, Json<Vec<JsonValue>>) {
    let db = _app_state.db.clone();
    let result: Vec<JsonValue> = db.select(collection).await.unwrap();
    (StatusCode::OK, Json(result))
}

#[derive(Debug, Serialize, Deserialize)]
struct EntityDocumentId {
    collection: String,
    id: String,
}

async fn get_entity(
    State(_app_state): State<AppState>,
    Path(doc_id): Path<EntityDocumentId>
) -> (StatusCode, Json<JsonValue>) {
    let db = _app_state.db.clone();
    let collection = doc_id.collection;
    let id = doc_id.id;
    let result: Option<JsonValue> = db.select((collection, id)).await.unwrap();
    let entity = result.unwrap();
    (StatusCode::OK, Json(entity))
}

async fn update_entity(
    State(_app_state): State<AppState>,
    Path(doc_id): Path<EntityDocumentId>,
    Json(payload): Json<JsonValue>,
) -> (StatusCode, Json<JsonValue>) {
    let db = _app_state.db.clone();
    let collection = doc_id.collection;
    let id = doc_id.id;
    let result: Option<JsonValue> = db.update((collection, id)).merge(payload).await.unwrap();
    let result = result.unwrap();
    (StatusCode::OK, Json(result))
}