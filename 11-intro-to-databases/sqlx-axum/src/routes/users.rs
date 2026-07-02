// src/routes/users.rs — HTTP handlers for /users. (TODO.)
//
// Each handler should be thin: extract from the request, call ONE repository
// function from `crate::db::users`, and map the result into `AppError`. No SQL
// lives here. Implement the bodies once `db/users.rs` is done, then uncomment
// the matching routes in `src/router.rs`.
//
// Required imports (uncomment when you start):
//
// use axum::{
//     extract::{Path, Query, State},
//     http::StatusCode,
//     Json,
// };
// use uuid::Uuid;
//
// use crate::db;
// use crate::error::AppError;
// use crate::models::{CreateUserRequest, ListQuery, UpdateUserRequest, User};
// use crate::state::AppState;

/// POST /users — create. Duplicate email → 409 Conflict.
pub async fn create() {
    // State(state): State<AppState>, Json(req): Json<CreateUserRequest>
    // -> Result<Json<User>, AppError>
    todo!("call db::users::create_user(&state.pool, &req); map unique-violation to AppError::Conflict")
}

/// GET /users/:id — fetch one; missing → 404.
pub async fn get() {
    // State(state): State<AppState>, Path(id): Path<Uuid>
    // -> Result<Json<User>, AppError>
    todo!("call db::users::get_user(&state.pool, id); None -> AppError::NotFound")
}

/// GET /users?limit=&offset= — paginated list (defaults 20/0, capped at 100).
pub async fn list() {
    // State(state): State<AppState>, Query(q): Query<ListQuery>
    // -> Result<Json<Vec<User>>, AppError>
    todo!("clamp q.limit to [1,100] and q.offset to >=0; call db::users::list_users")
}

/// PUT /users/:id — partial update (only fields provided). Missing id → 404.
pub async fn update() {
    // State(state): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<UpdateUserRequest>
    // -> Result<Json<User>, AppError>
    todo!("call db::users::update_user(&state.pool, id, req); None -> AppError::NotFound")
}

/// DELETE /users/:id — remove. Missing id → 404. Success → 204 No Content.
pub async fn delete() {
    // State(state): State<AppState>, Path(id): Path<Uuid>
    // -> Result<StatusCode, AppError>
    todo!("call db::users::delete_user; 0 rows -> AppError::NotFound; else StatusCode::NO_CONTENT")
}