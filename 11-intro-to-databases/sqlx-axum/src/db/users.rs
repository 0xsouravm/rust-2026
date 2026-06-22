// src/db/users.rs — sqlx repository functions for the User entity. (TODO.)
//
// This is the core of the lab. Implement one function per CRUD operation,
// each following this contract:
//
//   * borrow `&PgPool` (the shared pool)
//   * return `Result<_, sqlx::Error>` (handlers map to AppError via `?`)
//   * use `RETURNING` on INSERT/UPDATE so callers get generated columns
//     (id, timestamps) back in one round-trip
//   * use i64 for LIMIT/OFFSET (Postgres LIMIT is BIGINT)
//
// Use the `query_as!` / `query!` macros so every query is compile-time
// verified against the live schema in ./migrations (a wrong column name is a
// build error). NOTE: `query_as!` requires DATABASE_URL to point at a DB with
// the migrations applied *at compile time* — see README.md → "Build setup".
//
// fetch_one      → exactly one row (Err::RowNotFound on zero)
// fetch_optional → zero or one row (Option<User>)  ← use for get-by-id
// fetch_all      → zero or more rows (Vec<User>)    ← use for list
//
// Suggested signatures (uncomment and fill in the SQL):
//
// use sqlx::PgPool;
// use uuid::Uuid;
//
// use crate::models::{CreateUserRequest, UpdateUserRequest, User};
//
// pub async fn create_user(pool: &PgPool, req: &CreateUserRequest) -> Result<User, sqlx::Error> {
//     todo!("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING ...")
// }
//
// pub async fn get_user(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
//     todo!("SELECT ... FROM users WHERE id = $1  (fetch_optional)")
// }
//
// pub async fn list_users(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<User>, sqlx::Error> {
//     todo!("SELECT ... FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2  (fetch_all)")
// }
//
// pub async fn update_user(
//     pool: &PgPool,
//     id: Uuid,
//     req: UpdateUserRequest,
// ) -> Result<Option<User>, sqlx::Error> {
//     todo!("UPDATE users SET name = COALESCE($2, name), is_active = COALESCE($3, is_active) \
//            WHERE id = $1 RETURNING ...  (fetch_optional)")
// }
//
// pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
//     todo!("DELETE FROM users WHERE id = $1  → return rows_affected()")
// }