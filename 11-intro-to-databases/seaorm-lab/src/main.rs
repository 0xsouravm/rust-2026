// src/main.rs — entry point for the seaorm-lab Users + Posts API.
//
// Startup sequence:
//   1. load .env
//   2. init tracing
//   3. connect (ConnectOptions → tuned pool, sqlx_logging on)
//   4. run SeaORM migrations programmatically (Migrator::up)
//   5. build AppState + Router, serve

mod entities;
mod error;
mod handlers;
mod migration;
mod router;
mod state;

use sea_orm::{ConnectOptions, Database};
use tracing_subscriber::EnvFilter;

use migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sea_orm=warn".into()),
        )
        .init();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");

    let mut opt = ConnectOptions::new(url);
    opt.max_connections(20)
        .min_connections(2)
        .sqlx_logging(true); // log every generated SQL in dev

    let db = Database::connect(opt).await?;

    // Apply pending migrations from src/migration. A no-op if current.
    Migrator::up(&db, None).await?;

    let app = router::build_app(state::AppState { db });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3013").await?;
    tracing::info!("seaorm-lab listening on http://127.0.0.1:3013");
    axum::serve(listener, app).await?;
    Ok(())
}