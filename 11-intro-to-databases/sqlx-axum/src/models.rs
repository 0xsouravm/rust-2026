// src/models.rs — request/response shapes.
//
// `User` (the row model) is provided so the sample `/health` query and your
// CRUD handlers have a struct to map rows into. The request DTOs that your
// handlers will deserialize from JSON / the query string are left for you to
// define — see the TODO block below.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// The users row. `FromRow` maps a Postgres row to this struct by column name;
/// `Serialize` lets Axum return it directly as JSON.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// TODO(you): add the request/query DTOs your handlers need. Suggested shapes:
//
//   #[derive(Debug, Deserialize)]
//   pub struct CreateUserRequest {
//       pub name: String,
//       pub email: String,
//   }
//
//   /// Partial update — every field optional. The repository can use
//   /// `COALESCE($n, col)` so `None` means "leave this column alone".
//   #[derive(Debug, Deserialize, Default)]
//   pub struct UpdateUserRequest {
//       pub name: Option<String>,
//       pub is_active: Option<bool>,
//   }
//
//   /// `?limit=10&offset=20` — both optional, clamp in the handler.
//   #[derive(Debug, Deserialize, Default)]
//   pub struct ListQuery {
//       pub limit: Option<i64>,
//       pub offset: Option<i64>,
//   }
//
// You'll need `use serde::Deserialize;` at the top when you add them.