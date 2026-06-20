// 04_crud — Full Create / Read / Update / Delete with sqlx.
//
// The pattern every repository function follows:
//   * take `&PgPool` (borrow the shared pool, never own it)
//   * return `Result<T, sqlx::Error>` (DB ops can fail — constraint, network,
//     row-not-found)
//   * use RETURNING so the caller gets the generated id + timestamp back
//     without a second round-trip
//   * use i64 for LIMIT/OFFSET (Postgres's LIMIT is BIGINT)
//
// Requires: Postgres running, DATABASE_URL set, migrations applied.
//
// Run with:
//   cargo run --bin 04_crud

use db_foundations::{connect, entities::User};
use uuid::Uuid;

// CREATE — INSERT ... RETURNING the full new row.
async fn create_user(pool: &sqlx::PgPool, name: &str, email: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "INSERT INTO users (name, email) VALUES ($1, $2) \
         RETURNING id, name, email, bio, is_active, created_at",
        name,
        email
    )
    .fetch_one(pool)
    .await
}

// READ ALL — paginated, newest first.
async fn list_users(pool: &sqlx::PgPool, limit: i64, offset: i64) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users \
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(pool)
    .await
}

// READ ONE — by id, may be absent.
async fn get_user(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}

// UPDATE — RETURNING the updated row.
async fn update_user_name(pool: &sqlx::PgPool, id: Uuid, name: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "UPDATE users SET name = $1 WHERE id = $2 \
         RETURNING id, name, email, bio, is_active, created_at",
        name,
        id
    )
    .fetch_one(pool)
    .await
}

// DELETE — execute, report rows_affected (0 = nothing matched).
async fn delete_user(pool: &sqlx::PgPool, id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = connect().await?;

    // Create two users.
    let neo = create_user(&pool, "Neo", "neo@crud.example").await?;
    let trin = create_user(&pool, "Trinity", "trinity@crud.example").await?;
    println!("Created {} ({}) and {} ({})", neo.name, neo.id, trin.name, trin.id);

    // List (limit 10).
    let users = list_users(&pool, 10, 0).await?;
    println!("\nlist_users(10,0): {}", users.iter().map(|u| u.name.clone()).collect::<Vec<_>>().join(", "));

    // Get one.
    let fetched = get_user(&pool, neo.id).await?;
    println!("get_user(neo) → present? {}", fetched.is_some());

    // Update.
    let renamed = update_user_name(&pool, neo.id, "Thomas Anderson").await?;
    println!("update_user_name → now {}", renamed.name);

    // Delete.
    let removed = delete_user(&pool, neo.id).await?;
    println!("delete_user(neo) → rows_affected = {removed}");
    let removed2 = delete_user(&pool, trin.id).await?;
    println!("delete_user(trin) → rows_affected = {removed2}");

    // Duplicate-email demo: the UNIQUE constraint becomes a sqlx::Error you
    // map to HTTP 409 Conflict in a handler.
    let _ = create_user(&pool, "Dup", "dup@crud.example").await?;
    match create_user(&pool, "Dup Again", "dup@crud.example").await {
        Ok(_) => println!("\nDuplicate email inserted (unexpected!)"),
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                println!("\nDuplicate email rejected: {} (constraint violation)", db_err.code().unwrap_or_default());
            }
            // cleanup
            let _ = sqlx::query!("DELETE FROM users WHERE email = $1", "dup@crud.example")
                .execute(&pool)
                .await;
        }
    }
    Ok(())
}