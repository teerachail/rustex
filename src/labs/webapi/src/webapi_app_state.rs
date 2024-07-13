use std::sync::Arc;

use surrealdb::engine::local::{Db, Mem};
// use surrealdb::sql::Thing;
use surrealdb::Surreal;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Arc<Surreal<Db>>,
}

// pub type AppSharedState = std::sync::Arc<AppState>;

impl AppState {
    pub async fn new() -> surrealdb::Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("test").use_db("test").await?;
        let db = Arc::new(db);

        let app_state = Self {
            db,
        };

        Ok(app_state)
    }
}

// Implement Send and Sync manually (requires reasoning about thread safety)
// unsafe impl Send for AppState {}
// unsafe impl Sync for AppState {}