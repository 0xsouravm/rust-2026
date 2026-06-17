// src/routes/health.rs — Step 1 of the hands-on. The simplest possible
// axum handler so you can verify the server boots and the type plumbing
// works.

use axum::{Json, Router, routing::get};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "crud-starter" }))
}

#[allow(dead_code)] // alternate entry point — kept for the hands-on
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
