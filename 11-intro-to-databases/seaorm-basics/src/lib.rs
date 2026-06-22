//! Shared helpers
//!
//! `connect()` builds a tuned `DatabaseConnection` (a SeaORM wrapper around a
//! sqlx pool — cheap to clone, Arc-wrapped internally). `setup()` additionally
//! creates the `users` and `posts` tables these examples use, so each example
//! is self-contained and runnable without a separate migration step.

pub mod entities;

use std::time::Duration;

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr};

/// Connect with pool options + SQL logging on (dev only — turn off in prod).
pub async fn connect() -> Result<DatabaseConnection, DbErr> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");

    let mut opt = ConnectOptions::new(url);
    opt.max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true); // prints every generated SQL — your dev X-ray

    Database::connect(opt).await
}

/// Connect, then create the users + posts tables (idempotent). The examples
/// call `reset()` after this to start from a clean slate.
pub async fn setup() -> Result<DatabaseConnection, DbErr> {
    let db = connect().await?;

    // db.execute_unprepared(
    //     r#"CREATE TABLE IF NOT EXISTS users (
    //         id            SERIAL PRIMARY KEY,
    //         email         VARCHAR(255) NOT NULL UNIQUE,
    //         username      VARCHAR(255) NOT NULL,
    //         password_hash VARCHAR(255) NOT NULL,
    //         is_active     BOOLEAN NOT NULL DEFAULT true,
    //         created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
    //     )"#,
    // )
    // .await?;

    // db.execute_unprepared(
    //     r#"CREATE TABLE IF NOT EXISTS posts (
    //         id        SERIAL PRIMARY KEY,
    //         user_id   INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    //         title     VARCHAR(255) NOT NULL,
    //         body      TEXT NOT NULL,
    //         published BOOLEAN NOT NULL DEFAULT false
    //     )"#,
    // )
    // .await?;

    Ok(db)
}

/// Wipe rows so examples are reproducible across runs.
pub async fn reset(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute_unprepared("DELETE FROM posts").await?;
    db.execute_unprepared("DELETE FROM users").await?;
    Ok(())
}