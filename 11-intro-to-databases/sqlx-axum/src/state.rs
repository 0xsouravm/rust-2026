// src/state.rs — shared application state. (DONE — no changes needed.)
//
// PgPool is cheap to clone (it clones an inner Arc to the connection pool),
// so AppState can derive Clone and Axum hands a copy to every handler.

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}