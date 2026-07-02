// 02_sql_fundamentals — SELECT / WHERE / ORDER / LIMIT / INSERT / UPDATE /
// DELETE / RETURNING / JOINs, all against Postgres via sqlx.
//
// Every query here is a `query!` / `query_as!` macro: sqlx sends the SQL to
// Postgres's query planner at COMPILE TIME and checks it against the real
// schema. A typo in a column name is a build error, not a 2am crash.
//
// Requires: Postgres running, DATABASE_URL set, migrations applied
// (db_foundations::connect() runs them for you).
//
// Run with:
//   cargo run --bin 02_sql_fundamentals

use db_foundations::{connect, entities::User};

// A purpose-built shape for the JOIN examples — the columns we SELECT, aliased
// to the field names. `FromRow` maps by name.
#[derive(Debug, sqlx::FromRow)]
struct UserPost {
    user_name: String,
    post_title: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = connect().await?;
    seed(&pool).await?;

    // ── SELECT: specific columns (never `*` in production) ──────────────────
    let users: Vec<User> = sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users \
         ORDER BY created_at ASC"
    )
    .fetch_all(&pool)
    .await?;
    println!("All users ({}):", users.len());
    for u in &users {
        println!("  - {} <{}> active={}", u.name, u.email, u.is_active);
    }

    // ── WHERE + AND ────────────────────────────────────────────────────────
    let active: Vec<User> = sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users \
         WHERE is_active = $1 AND created_at > NOW() - INTERVAL '1 hour'",
        true
    )
    .fetch_all(&pool)
    .await?;
    println!("\nActive users created in the last hour: {}", active.len());

    // ── ORDER BY + LIMIT/OFFSET (pagination) ───────────────────────────────
    // LIMIT/OFFSET are BIGINT → i64 in Rust. This is page 1 (offset 1).
    let page: Vec<User> = sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users \
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        2i64,
        1i64
    )
    .fetch_all(&pool)
    .await?;
    println!("\nPage 2 (limit 2, offset 1):");
    for u in &page {
        println!("  - {}", u.name);
    }

    // ── LIKE (pattern matching, % is the wildcard) ────────────────────────
    let gmail: Vec<User> = sqlx::query_as!(
        User,
        "SELECT id, name, email, bio, is_active, created_at FROM users \
         WHERE email LIKE $1",
        "%@gmail.com%"
    )
    .fetch_all(&pool)
    .await?;
    println!("\nUsers with a gmail-ish email: {}", gmail.len());

    // ── COUNT(*) (aggregate) — alias to name the anonymous column ──────────
    let total = sqlx::query!("SELECT COUNT(*) AS \"count\" FROM users")
        .fetch_one(&pool)
        .await?;
    println!("\nTotal users: {}", total.count);

    // ── INSERT ... RETURNING (Postgres gives the generated id/ts back) ─────
    let created: User = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email) VALUES ($1, $2) \
         RETURNING id, name, email, bio, is_active, created_at",
        "Tank",
        "tank@realworld.dev"
    )
    .fetch_one(&pool)
    .await?;
    println!("\nInserted Tank, db assigned id = {}", created.id);

    // ── UPDATE ... RETURNING (confirm what changed) ────────────────────────
    let updated: User = sqlx::query_as!(
        User,
        "UPDATE users SET is_active = $1 WHERE email = $2 \
         RETURNING id, name, email, bio, is_active, created_at",
        false,
        "tank@realworld.dev"
    )
    .fetch_one(&pool)
    .await?;
    println!("Deactivated {} (is_active now {})", updated.name, updated.is_active);

    // ── DELETE — execute, read rows_affected ───────────────────────────────
    let del = sqlx::query!("DELETE FROM users WHERE email = $1", "tank@realworld.dev")
        .execute(&pool)
        .await?;
    println!("Deleted {} row(s)", del.rows_affected());

    // ── INNER JOIN (only matching rows on both sides) ──────────────────────
    let inner: Vec<UserPost> = sqlx::query_as!(
        UserPost,
        "SELECT u.name AS user_name, p.title AS post_title \
         FROM users u \
         INNER JOIN posts p ON p.user_id = u.id \
         ORDER BY u.name, p.title"
    )
    .fetch_all(&pool)
    .await?;
    println!("\nINNER JOIN (users who have posts):");
    for r in &inner {
        println!("  - {} wrote \"{}\"", r.user_name, r.post_title);
    }

    // ── LEFT JOIN (all users, posts if any — NULL where none) ──────────────
    // The post title is nullable on the left side, so we read it as Option.
    let left = sqlx::query!(
        "SELECT u.name AS user_name, p.title AS post_title \
         FROM users u \
         LEFT JOIN posts p ON p.user_id = u.id \
         ORDER BY u.name"
    )
    .fetch_all(&pool)
    .await?;
    println!("\nLEFT JOIN (every user, with/without posts):");
    for r in &left {
        let title = r.post_title.unwrap_or_else(|| "—".to_string());
        println!("  - {} → {}", r.user_name, title);
    }

    Ok(())
}

/// Wipe and reseed users + posts so the example is reproducible.
async fn seed(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM posts").execute(pool).await?;
    sqlx::query!("DELETE FROM users").execute(pool).await?;

    sqlx::query!(
        "INSERT INTO users (name, email) VALUES ($1, $2), ($3, $4), ($5, $6)",
        "Neo", "neo@matrix.gmail.com",
        "Trinity", "trinity@matrix.com",
        "Oracle", "oracle@matrix.com",
    )
    .execute(pool)
    .await?;

    // Give Neo two posts, Trinity one, Oracle none (so LEFT JOIN shows a NULL).
    let neo = sqlx::query!("SELECT id FROM users WHERE email = $1", "neo@matrix.gmail.com")
        .fetch_one(pool)
        .await?;
    let trinity = sqlx::query!("SELECT id FROM users WHERE email = $1", "trinity@matrix.com")
        .fetch_one(pool)
        .await?;

    sqlx::query!(
        "INSERT INTO posts (user_id, title, body) VALUES \
         ($1, $2, $3), ($1, $4, $5), ($6, $7, $8)",
        neo.id, "Red Pill Guide", "Take the red pill.",
        neo.id, "Matrix Survival", "There is no spoon.",
        trinity.id, "Hacking 101", "Learn to fly a helicopter in 10s.",
    )
    .execute(pool)
    .await?;

    Ok(())
}