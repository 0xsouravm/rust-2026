// 01_connect — Establishing a SeaORM connection.
//
// SeaORM wraps a sqlx connection pool in `DatabaseConnection`. Like `PgPool`,
// it's cheap to clone (Arc-wrapped internally) and safe to share across async
// tasks — which is why it can live directly in Axum's AppState.
//
// `ConnectOptions` lets you tune the pool and, importantly, turn on
// `sqlx_logging` so every generated SQL statement is logged in dev.
//
// Needs DATABASE_URL. Run with:
//   cargo run --bin 01_connect

use sea_orm::{ConnectionTrait, DbErr};

use seaorm_basics::connect;

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = connect().await?;
    println!("Connected to Postgres via SeaORM.");
    println!("Backend in use: {:?}", db.get_database_backend());
    println!("Pool tuned in seaorm_basics::connect (max=20, sqlx_logging on).");
    println!("DatabaseConnection is Clone (Arc-wrapped) → drop it straight into AppState.");
    Ok(())
}