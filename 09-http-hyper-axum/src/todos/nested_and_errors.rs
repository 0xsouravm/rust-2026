// Nested routers and a custom error type
//
// Concepts:
//   - `.nest("/prefix", sub_router)` — compose routers with a path prefix.
//   - `.merge(other_router)` — combine routers at the same path level.
//   - A custom error enum that implements `IntoResponse`.
//   - `Result<Ok, AppError>` returns — let the error type carry the HTTP
//     status, not the handler.
//
// Run with:
//   cargo run --bin nested_and_errors
//   curl http://localhost:3004/health
//   curl http://localhost:3004/api/v1/notes
//   curl -X POST -H 'Content-Type: application/json' \
//        -d '{"text":"hello"}' http://localhost:3004/api/v1/notes
//   curl http://localhost:3004/api/v1/notes/1
//   curl http://localhost:3004/api/v1/notes/999   # AppError::NotFound

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;

// ── AppError ──────────────────────────────────────────────────────────
//
// One enum, one IntoResponse impl, and every handler in the app can
// return Result<T, AppError>. The match inside `into_response` is the
// ONE place that maps domain errors to HTTP responses.

#[derive(Debug)]
// `Internal` is reserved for future infrastructure failures (DB down,
// upstream timeout, etc.). Demonstrates how the enum scales.
#[allow(dead_code)]
enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(what)  => (StatusCode::NOT_FOUND,            what),
            AppError::BadRequest(why) => (StatusCode::BAD_REQUEST,          why),
            AppError::Internal(why)   => (StatusCode::INTERNAL_SERVER_ERROR, why),
        };

        // Build a uniform JSON error body. Clients can switch on `error`.
        let body = Json(serde_json::json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

// Convenience: any `String` we want to throw becomes a BadRequest.
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::BadRequest(s)
    }
}

// ── State (kept tiny here — the focus is on routing + errors) ─────────

#[derive(Clone)]
struct AppState {
    notes: Arc<RwLock<HashMap<u64, Note>>>,
    next_id: Arc<RwLock<u64>>,
}

#[derive(Serialize, Clone)]
struct Note {
    id: u64,
    text: String,
}

#[derive(Deserialize)]
struct NewNote {
    text: String,
}

// ── Notes module ──────────────────────────────────────────────────────
//
// Each domain gets its own router-returning function. Paths inside are
// RELATIVE — the prefix is added by the parent via `.nest(...)`.

mod notes {
    use super::*;

    async fn list(State(state): State<AppState>) -> Json<Vec<Note>> {
        let notes = state.notes.read().unwrap();
        Json(notes.values().cloned().collect())
    }

    async fn create(
        State(state): State<AppState>,
        Json(payload): Json<NewNote>,
    ) -> Result<(StatusCode, Json<Note>), AppError> {
        if payload.text.trim().is_empty() {
            return Err(AppError::BadRequest("text cannot be empty".into()));
        }

        let id = {
            let mut next = state.next_id.write().unwrap();
            let id = *next;
            *next += 1;
            id
        };

        let note = Note { id, text: payload.text };
        state.notes.write().unwrap().insert(id, note.clone());

        Ok((StatusCode::CREATED, Json(note)))
    }

    async fn get_one(
        State(state): State<AppState>,
        Path(id): Path<u64>,
    ) -> Result<Json<Note>, AppError> {
        let notes = state.notes.read().unwrap();
        notes
            .get(&id)
            .cloned()
            .map(Json)
            .ok_or_else(|| AppError::NotFound(format!("note {id} not found")))
    }

    pub fn router() -> Router<AppState> {
        Router::new()
            .route("/",      get(list).post(create))
            // axum 0.8 uses `{id}` (not `:id`) for path parameters.
            .route("/{id}",  get(get_one))
    }
}

// ── A health route at the top level (no prefix) ───────────────────────

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

// ── Build the app ─────────────────────────────────────────────────────
//
// Notice the two ways routers combine:
//   - `.nest("/api/v1/notes", notes::router())` — adds a prefix.
//   - `.route("/health", get(health))` — same router, no prefix.
//
// The whole builder carries the `AppState` type parameter from the
// sub-router up — that's why `build_router` returns `Router<AppState>`.
// `with_state` is called at the very end to attach the actual value.

fn build_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .nest("/api/v1/notes", notes::router())
}

fn app_state() -> AppState {
    AppState {
        notes:   Arc::new(RwLock::new(HashMap::new())),
        next_id: Arc::new(RwLock::new(1)),
    }
}

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:3004".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("listening on http://{addr}");
    println!();
    println!("Try:");
    println!("  curl http://{addr}/health");
    println!("  curl http://{addr}/api/v1/notes");
    println!("  curl -X POST -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"text\":\"Wake up, Neo...\"}}' \\");
    println!("       http://{addr}/api/v1/notes");
    println!("  curl http://{addr}/api/v1/notes/1");
    println!("  curl -i http://{addr}/api/v1/notes/999   # 404 via AppError");
    println!("  curl -X POST -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"text\":\"   \"}}' http://{addr}/api/v1/notes   # 400 via AppError");

    let app = build_router().with_state(app_state());
    axum::serve(listener, app).await.unwrap();
}
