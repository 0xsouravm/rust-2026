// 06_migrations — Version control for your schema.
//
// Migrations are how you change a production schema safely: timestamped SQL
// files, applied once, in order, tracked in the `_sqlx_migrations` table.
// Never edit a migration that's already been applied — the stored checksum
// will mismatch and sqlx will refuse to run. Add a *new* migration instead.
//
// Two ways to run them:
//   1. CLI (dev):      `sqlx migrate add <name>` then `sqlx migrate run`
//   2. Embedded (app): `sqlx::migrate!("./migrations").run(&pool).await?`
//      This is what `db_foundations::connect()` does — the files are baked
//      into the binary at compile time, so `cargo run` is self-contained and
//      containers start with the schema in sync.
//
// Requires: Postgres running, DATABASE_URL set.
//
// Run with:
//   cargo run --bin 06_migrations

use db_foundations::connect;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = connect().await?; // runs pending migrations, then we list them

    let applied = sqlx::query!(
        "SELECT version, description FROM _sqlx_migrations ORDER BY version"
    )
    .fetch_all(&pool)
    .await?;

    println!("Applied migrations ({} total):", applied.len());
    for m in &applied {
        println!("  {:>14}  {}", m.version, m.description);
    }

    println!();
    println!("Equivalent CLI workflow:");
    println!("  sqlx migrate add add_bio_to_users        # writes a new timestamped file");
    println!("  $EDITOR migrations/<timestamp>_add_bio_to_users.sql");
    println!("  sqlx migrate run                          # applies pending");
    println!("  sqlx migrate info                         # shows status");
    println!();
    println!("Embed them so `cargo run` self-migrates:");
    println!("  sqlx::migrate!(\"./migrations\").run(&pool).await?;");
    println!();
    println!("Offline builds / CI (no live DB at build time):");
    println!("  cargo install sqlx-cli && cargo sqlx prepare   # writes .sqlx/ cache");
    println!("  SQLX_OFFLINE=true cargo build                   # uses the cache");
    Ok(())
}