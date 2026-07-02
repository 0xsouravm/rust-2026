// src/main.rs — entry point for the sqlx-axum starter.
//
// This file is DONE for you: the standard Axum + sqlx startup sequence.
// Your job is the CRUD layer (db/users.rs + routes/users.rs) — see the TODO
// list in README.md.
//
// Startup sequence (the same one every real Axum+sqlx app follows):
//   1. load .env            (dotenvy)
//   2. init tracing         (so we get request logs)
//   3. connect a PgPool
//   4. run migrations       (sqlx::migrate!  — schema baked into the binary)
//   5. build AppState + Router, serve

mod db;
mod error;
mod models;
mod router;
mod routes;
mod state;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&url)
        .await?;

    // Apply pending migrations from ./migrations on every startup. If the
    // schema is already current this is a no-op (it checks _sqlx_migrations).
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = router::build_app(state::AppState { pool });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3014").await?;
    tracing::info!("sqlx-axum listening on http://127.0.0.1:3014");
    axum::serve(listener, app).await?;
    Ok(())
}