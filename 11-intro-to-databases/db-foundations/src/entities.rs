//! Row models shared across the examples.
//!
//! `#[derive(sqlx::FromRow)]` tells sqlx how to map a database row to the
//! struct — column names must match field names. `Option<T>` maps `NULL`.
//! We derive `Serialize` so the lab can hand these straight to Axum as JSON.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Post {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub created_at: DateTime<Utc>,
}

/// Account row used by the transaction example.
#[derive(Debug, Clone, FromRow)]
pub struct Account {
    pub user_id: Uuid,
    pub credits: i64,
}