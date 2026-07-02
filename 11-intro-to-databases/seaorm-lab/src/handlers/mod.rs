// src/handlers/mod.rs — re-exports sub-handlers + the public health handler.

pub mod users;

use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "seaorm-lab" }))
}