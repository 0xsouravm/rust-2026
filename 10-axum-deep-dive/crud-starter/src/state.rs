// src/state.rs — the in-memory user store.
//
// The `db` and `next_id` are wrapped in `Arc<RwLock<_>>` so they can
// be cloned cheaply into every handler via `State<AppState>`.
//
// Real code would back this with sqlx + Postgres. The HTTP layer
// wouldn't change.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::models::User;

#[derive(Clone)]
pub struct AppState {
    pub db:      Arc<RwLock<HashMap<u64, User>>>,
    pub next_id: Arc<RwLock<u64>>,
}

impl AppState {
    pub fn new() -> Self {
        let mut db = HashMap::new();
        db.insert(1, User { id: 1, name: "Neo".into(),     email: "neo@zion.io".into() });
        db.insert(2, User { id: 2, name: "Trinity".into(), email: "tri@zion.io".into() });

        Self {
            db:      Arc::new(RwLock::new(db)),
            next_id: Arc::new(RwLock::new(3)),
        }
    }

    pub fn allocate_id(&self) -> u64 {
        let mut next = self.next_id.write().unwrap();
        let id = *next;
        *next += 1;
        id
    }
}
