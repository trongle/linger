use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn routes() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(health))
        .route("/ready", get(health))
}
async fn health() -> Json<Value> {
    Json(json!({"status":"ok"}))
}
