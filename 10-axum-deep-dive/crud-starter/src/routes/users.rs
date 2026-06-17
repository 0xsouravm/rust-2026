// src/routes/users.rs — CRUD handlers + the users sub-router.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, MethodRouter},
    Json, Router,
};

use crate::{
    error::AppError,
    models::{ListQuery, NewUser, UpdateUser, User},
    state::AppState,
};

fn require_nonempty(field: &str, value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(AppError::BadRequest(format!("{field} cannot be empty")))
    } else {
        Ok(trimmed.to_string())
    }
}

// GET /users?search=...&limit=...
async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<User>>, AppError> {
    let db = state.db.read().unwrap();
    let mut users: Vec<User> = db.values().cloned().collect();

    if let Some(q) = params.search.as_deref().map(str::to_lowercase) {
        users.retain(|u| u.name.to_lowercase().contains(&q));
    }
    if let Some(limit) = params.limit {
        users.truncate(limit);
    }
    Ok(Json(users))
}

// GET /users/{id}
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<User>, AppError> {
    let db = state.db.read().unwrap();
    db.get(&id).cloned().map(Json)
        .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))
}

// POST /users
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<NewUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let name  = require_nonempty("name",  &payload.name)?;
    let email = require_nonempty("email", &payload.email)?;
    let id    = state.allocate_id();
    let user  = User { id, name, email };
    state.db.write().unwrap().insert(id, user.clone());
    Ok((StatusCode::CREATED, Json(user)))
}

// PATCH /users/{id} — partial update
async fn patch_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    let mut db = state.db.write().unwrap();
    let user = db
        .get_mut(&id)
        .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;
    if let Some(name)  = &payload.name  { user.name  = require_nonempty("name",  name)?;  }
    if let Some(email) = &payload.email { user.email = require_nonempty("email", email)?; }
    Ok(Json(user.clone()))
}

// DELETE /users/{id}
async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, AppError> {
    match state.db.write().unwrap().remove(&id) {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None    => Err(AppError::NotFound(format!("user {id} not found"))),
    }
}

/// GET/PATCH/DELETE /users/{id} — returned as a MethodRouter so the
/// parent can stack timing + auth on JUST this branch.
pub fn read_user_route() -> MethodRouter<AppState> {
    get(get_user).patch(patch_user).delete(delete_user)
}

/// GET /users — list, used by the v1 merge.
pub fn list_route() -> MethodRouter<AppState> {
    get(list_users)
}

/// Sub-router exposing every CRUD endpoint.  Kept as a reference for
/// the hands-on — the main app wires routes individually in `router.rs`.
#[allow(dead_code)]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",       get(list_users).post(create_user))
        .route("/{id}",   read_user_route())
}
