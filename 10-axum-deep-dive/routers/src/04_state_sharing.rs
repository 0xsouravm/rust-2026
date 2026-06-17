// 04_state_sharing — Application state via State<T>
//
// What this demonstrates:
//   - Defining a Clone-able AppState
//   - `Arc<RwLock<T>>` for shared, mutable, async-safe data
//   - `.with_state(state)` to attach state to a router
//   - `State(state): State<AppState>` extractor in handlers
//   - Why State is a `FromRequestParts` extractor (must come before body)
//
// Run with:
//   cargo run --bin 04_state_sharing
//   curl http://127.0.0.1:3013/visits
//   curl -X POST http://127.0.0.1:3013/visits
//   curl http://127.0.0.1:3013/visits
//   curl -X DELETE http://127.0.0.1:3013/visits   # reset

#[allow(unused_imports)] // post is used in the method chain `.post(record_visit)`
use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;

// AppState MUST be Clone — Axum clones it per request
#[derive(Clone)]
struct AppState {
    // Read-heavy data: RwLock gives concurrent reads, exclusive writes
    visits: Arc<RwLock<HashMap<String, u64>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            visits: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

async fn list_visits(State(state): State<AppState>) -> Json<serde_json::Value> {
    let map = state.visits.read().unwrap();
    Json(json!({ "visits": map.clone() }))
}

async fn record_visit(State(state): State<AppState>) -> StatusCode {
    // Use a write lock briefly, then drop it
    {
        let mut map = state.visits.write().unwrap();
        *map.entry("total".to_string()).or_insert(0) += 1;
    }
    StatusCode::CREATED
}

async fn reset_visits(State(state): State<AppState>) -> StatusCode {
    state.visits.write().unwrap().clear();
    StatusCode::NO_CONTENT
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/visits", get(list_visits).post(record_visit).delete(reset_visits))
        .with_state(AppState::new());

    let addr: SocketAddr = "127.0.0.1:3013".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("04_state_sharing listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
