// src/handlers/users.rs — HTTP handlers for /users (and /users/{id}/posts).
//
// extract from the request, call SeaORM, map the result into AppError

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sea_orm::{
    ActiveModelTrait, EntityTrait, ModelTrait, Order, PaginatorTrait, QueryOrder, Set,
};

use crate::entities::{post, user};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateUserDto {
    pub username: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub page: u64,
    pub total_pages: u64,
}

// ── Handlers ──────────────────────────────────────────────────────────────

/// GET /users?page=&page_size= — paginated list. page_size capped at 100.
pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<user::Model>>, AppError> {
    let page = p.page.unwrap_or(0);
    let page_size = p.page_size.unwrap_or(20).clamp(1, 100);

    let paginator = user::Entity::find()
        .order_by(user::Column::CreatedAt, Order::Desc)
        .paginate(&state.db, page_size);
    let total_pages = paginator.num_pages().await?;
    let users = paginator.fetch_page(page).await?;

    Ok(Json(PaginatedResponse { data: users, page, total_pages }))
}

/// POST /users — create. Duplicate email → 409 (mapped in AppError).
pub async fn create(
    State(state): State<AppState>,
    Json(dto): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<user::Model>), AppError> {
    // NOTE: storing the password verbatim is for the lab only — hash it
    // (Argon2) in production. The field is `#[serde(skip)]` on the entity so
    // it never leaks back out in responses.
    let new_user = user::ActiveModel {
        email: Set(dto.email),
        username: Set(dto.username),
        password_hash: Set(format!("plain:{}", dto.password)),
        is_active: Set(true),
        ..Default::default()
    };
    let created = new_user.insert(&state.db).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

/// GET /users/{id} — fetch one; missing → 404.
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<user::Model>, AppError> {
    user::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))
        .map(Json)
}

/// PUT /users/{id} — partial update (only the fields provided). Missing → 404.
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<user::Model>, AppError> {
    let existing = user::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;

    let mut am: user::ActiveModel = existing.into();
    if let Some(username) = dto.username {
        am.username = Set(username);
    }
    if let Some(is_active) = dto.is_active {
        am.is_active = Set(is_active);
    }
    Ok(Json(am.update(&state.db).await?))
}

/// DELETE /users/{id} — remove. Missing → 404. Success → 204.
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let res = user::Entity::delete_by_id(id).exec(&state.db).await?;
    if res.rows_affected == 0 {
        return Err(AppError::NotFound(format!("user {id} not found")));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// GET /users/{id}/posts — the related posts for a user (find_with_related's
/// lazy sibling, find_related). Missing user → 404.
pub async fn user_posts(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<post::Model>>, AppError> {
    let u = user::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;
    let posts = u.find_related(post::Entity).all(&state.db).await?;
    Ok(Json(posts))
}