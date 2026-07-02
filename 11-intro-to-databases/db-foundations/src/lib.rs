//! Shared helpers for the db-foundations examples.
//!
//! Every example (except `01_why_databases`) needs the same three setup steps:
//!   1. load `DATABASE_URL` from `.env`,
//!   2. connect a [`PgPool`] with sane pool options,
//!   3. run the SQL in `migrations/` so the schema the `query_as!` macros are
//!      checked against at compile time actually exists at runtime.
//!
//! Doing it here keeps each example focused on the SQL concept it teaches.

pub mod entities;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Connect a pool and apply pending migrations from `./migrations`.
///
/// `sqlx::migrate!("./migrations")` embeds the migration files into the binary
/// at compile time (path resolved relative to this crate's `Cargo.toml`), so
/// `cargo run` is self-contained — no `sqlx migrate run` needed separately.
pub async fn connect() -> anyhow::Result<PgPool> {
    dotenvy::dotenv().ok();

    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set — copy .env.example to .env");

    let pool = PgPoolOptions::new()
        .max_connections(10) // ceiling; the pool grows under load up to this
        .min_connections(1) // keep one warm so the first request isn't slow
        .acquire_timeout(Duration::from_secs(3)) // fail fast if exhausted
        .connect(&url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}