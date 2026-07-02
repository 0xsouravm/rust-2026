// src/routes/mod.rs — re-exports sub-routers + the public health handler.
//
// `/health` is DONE and is your proof that sqlx is wired up: it runs a real
// query against the pool and reports the row count. Once `cargo run` shows
// `"users": N` here, you know the pool + migrations work and you can start
// building the /users TODO with confidence.

pub mod users;

use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

/// GET /health — ping the database and report the user count.
///
/// This uses the *runtime* `sqlx::query_as` (not the `query_as!` macro) so the
/// starter compiles without a live database present at build time. Your CRUD
/// code should prefer the `query_as!` macro for compile-time SQL verification.
pub async fn health(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(json!({ "status": "ok", "service": "sqlx-axum", "users": count })))
}