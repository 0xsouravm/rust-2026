use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
    routing::{get, MethodRouter},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{state::AppState, error::AppError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
}

fn require_nonempty(field: &str, value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{field} cannot be empty")));
    }
    Ok(trimmed.to_string())
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<UpdateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let name = payload.name.ok_or_else(|| AppError::BadRequest("name required".to_string()))?;
    let email = payload.email.ok_or_else(|| AppError::BadRequest("email required".to_string()))?;
    let name_clean = require_nonempty("name", &name)?;
    let email_clean = require_nonempty("email", &email)?;
    let mut db = state.db.write().unwrap();
    let next_id = db.keys().max().copied().unwrap_or(0) + 1;
    let user = User { id: next_id, name: name_clean, email: email_clean };
    db.insert(next_id, user.clone());
    Ok((StatusCode::CREATED, Json(user)))
}

async fn get_user(State(state): State<AppState>, Path(id): Path<u64>) -> Result<Json<User>, AppError> {
    let db = state.db.read().unwrap();
    let user = db.get(&id).ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;
    Ok(Json(user.clone()))
}

async fn patch_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    let mut db = state.db.write().unwrap();
    let user = db.get_mut(&id).ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;
    if let Some(name) = &payload.name { user.name = require_nonempty("name", name)?; }
    if let Some(email) = &payload.email { user.email = require_nonempty("email", email)?; }
    Ok(Json(user.clone()))
}

async fn put_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    let mut db = state.db.write().unwrap();
    let user = db.get_mut(&id).ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;
    if let Some(name) = &payload.name { user.name = require_nonempty("name", name)?; }
    if let Some(email) = &payload.email { user.email = require_nonempty("email", email)?; }
    Ok(Json(user.clone()))
}

async fn delete_user(State(state): State<AppState>, Path(id): Path<u64>) -> Result<StatusCode, AppError> {
    let mut db = state.db.write().unwrap();
    db.remove(&id).ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let db = state.db.read().unwrap();
    let list: Vec<User> = db.values().cloned().collect();
    Json(list)
}

pub fn read_user_route() -> MethodRouter<AppState> {
    get(get_user).patch(patch_user).put(put_user).delete(delete_user)
}

pub fn list_route() -> MethodRouter<AppState> {
    get(list_users)
}
