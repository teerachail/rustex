use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put, Router},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use surrealdb::{
    opt::{PatchOp, RecordId},
    sql::statements::{BeginStatement, CommitStatement},
};

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
        // POST /api/:collection/:id/:sub_entity goes to `patch_add_sub_entity`
        .route("/:collection/:id/:sub_entity", post(patch_add_sub_entity))
        // POST /api/txs goes to `execute_txs`
        .route("/txs", post(execute_txs))
        .with_state(app_state)
}

async fn test(State(_app_state): State<AppState>) -> (StatusCode, Json<JsonValue>) {
    (StatusCode::OK, Json(json!({"message": "Hello, World!"})))
}

#[derive(Debug, Serialize, Deserialize)]
struct ResultForRecordId {
    #[allow(dead_code)]
    id: RecordId,
}

fn convert_object_id_to_string(value: JsonValue) -> Result<JsonValue, serde_json::Error> {
    match value {
        JsonValue::Object(mut obj) => {
            if let Some(record) = obj.remove("id") {
                let id: RecordId = serde_json::from_value(record).unwrap();
                obj.insert("id".to_owned(), JsonValue::String(id.id.to_string()));
            }
            Ok(JsonValue::Object(obj))
        }
        _ => Ok(value), // Return the original value for non-objects
    }
}

async fn create_entity(
    State(_app_state): State<AppState>,
    Path(collection): Path<String>,
    Json(payload): Json<JsonValue>,
) -> (StatusCode, Json<JsonValue>) {
    let db = _app_state.db.clone();
    let result: Vec<ResultForRecordId> = db.create(collection).content(payload).await.unwrap();
    let result = result.first().unwrap();
    let id = result.id.id.to_string();
    (StatusCode::OK, Json(json!({"id": id})))
}

async fn list_entities(
    State(_app_state): State<AppState>,
    Path(collection): Path<String>,
) -> (StatusCode, Json<Vec<JsonValue>>) {
    let db = _app_state.db.clone();
    let result: Vec<JsonValue> = db.select(collection).await.unwrap();
    let result: Vec<JsonValue> = result
        .iter()
        .map(|x| convert_object_id_to_string(x.clone()).unwrap())
        .collect();
    (StatusCode::OK, Json(result))
}

#[derive(Debug, Serialize, Deserialize)]
struct EntityDocumentId {
    collection: String,
    id: String,
}

async fn get_entity(
    State(_app_state): State<AppState>,
    Path(doc_id): Path<EntityDocumentId>,
) -> (StatusCode, Json<JsonValue>) {
    let db = _app_state.db.clone();
    let collection = doc_id.collection;
    let id = doc_id.id;
    let result: Option<JsonValue> = db.select((collection, id)).await.unwrap();
    if let Some(entity) = result {
        let entity = convert_object_id_to_string(entity).unwrap();
        (StatusCode::OK, Json(entity))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
    }
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
    let result = convert_object_id_to_string(result).unwrap();
    (StatusCode::OK, Json(result))
}

#[derive(Debug, Serialize, Deserialize)]
struct EntitySubDocumentId {
    collection: String,
    id: String,
    sub_entity: String,
}

async fn patch_add_sub_entity(
    State(_app_state): State<AppState>,
    Path(doc_id): Path<EntitySubDocumentId>,
    Json(payload): Json<JsonValue>,
) -> (StatusCode, Json<JsonValue>) {
    let db = _app_state.db.clone();
    let collection = doc_id.collection;
    let id = doc_id.id;
    let sub_entity = doc_id.sub_entity;
    let result: Option<JsonValue> = db
        .update((collection, id))
        .patch(PatchOp::add(&sub_entity, &[payload]))
        .await
        .unwrap();
    let result = result.unwrap();
    let result = convert_object_id_to_string(result).unwrap();
    (StatusCode::OK, Json(result))
}

#[derive(Debug, Serialize, Deserialize)]
struct Tx {
    id: String,
    field: String,
    amount: i64,
}

async fn execute_txs(
    State(_app_state): State<AppState>,
    Json(payload): Json<Vec<Tx>>,
) -> (StatusCode, Json<JsonValue>) {
    let db = _app_state.db.clone();
    let mut query = db.query(BeginStatement::default());

    for rec in payload.iter() {
        let op = if rec.amount >= 0 { "+=" } else { "-=" };
        let sql = format!("UPDATE {:} SET {:} {} $amount", rec.id, rec.field, op);
        println!("SQL: {:}", sql);
        query = query.query(sql).bind(rec);
    }

    let result = query.query(CommitStatement::default()).await.unwrap();
    let n = result.num_statements();

    (StatusCode::OK, Json(json!({"n": n })))
}
