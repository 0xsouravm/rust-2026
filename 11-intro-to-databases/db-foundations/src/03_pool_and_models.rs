// 03_pool_and_models — Connection pooling and the FromRow model.
//
// Opening a Postgres connection is expensive (TCP + auth + SSL). A pool keeps
// a fixed set of connections warm and hands them out per request. `PgPool`
// is `Clone` (clones the inner Arc), so one pool can be shared across
// every Axum handler via `State`.
//
// `FromRow` is the trait that maps a row to your struct by column name.
// `query_as!` builds on it: the macro checks the columns/types against the
// DB at compile time AND the result is mapped into your struct at runtime.
//
// fetch_one    → exactly one row (Err::RowNotFound on zero, Err on many)
// fetch_optional → zero or one row (Option<User>)
// fetch_all    → zero or more rows (Vec<User>)
//
// Requires: Postgres running, DATABASE_URL set, migrations applied.
//
// Run with:
//   cargo run --bin 03_pool_and_models

use std::time::Duration;

use db_foundations::{connect, entities::User};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = connect().await?;

    // Show the pool options in full — this is what you'd tune for production.
    // (db_foundations::connect already used a smaller version of this.)
    let url = std::env::var("DATABASE_URL")?;
    let _production_pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Some(Duration::from_secs(600)))
        .connect(&url)
        .await?;
    println!("Pool tuned: max=20, min=2, acquire_timeout=3s");

    // Insert a user so we have a known id to look up.
    let created = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email) VALUES ($1, $2) \
         RETURNING id, name, email, bio, is_active, created_at",
        "Morpheus",
        "morpheus@pool.example"
    )
    .fetch_one(&pool)
    .await?;

    // fetch_optional — zero or one. The canonical "get by id" shape.
    let maybe: Option<User> = sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users \
         WHERE id = $1",
        created.id
    )
    .fetch_optional(&pool)
    .await?;
    println!("\nfetch_optional by real id  → {:?}", maybe.as_ref().map(|u| &u.name));

    let missing_id = Uuid::new_v4();
    let none: Option<User> = sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users \
         WHERE id = $1",
        missing_id
    )
    .fetch_optional(&pool)
    .await?;
    println!("fetch_optional by fake id  → is_none? {}", none.is_none());

    // fetch_one — exactly one. Errors if the row is missing; map that to 404
    // in a handler.
    let one = sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users \
         WHERE id = $1",
        created.id
    )
    .fetch_one(&pool)
    .await?;
    println!("fetch_one                  → {}", one.name);

    // Cleanup so re-runs stay tidy.
    sqlx::query!("DELETE FROM users WHERE id = $1", created.id)
        .execute(&pool)
        .await?;
    Ok(())
}