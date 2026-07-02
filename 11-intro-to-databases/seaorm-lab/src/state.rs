// src/state.rs — shared application state.
//
// DatabaseConnection is Arc-wrapped internally, so cloning it (which AppState
// deriving Clone forces) just bumps a refcount — every Axum handler gets a
// cheap shared copy.

use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}