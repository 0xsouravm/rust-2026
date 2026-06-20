// 05_transactions — All-or-nothing with Rust's Drop doing the rollback.
//
// A transfer must deduct from Alice AND credit Bob, or neither. We BEGIN,
// run both UPDATEs on the transaction's connection (`&mut *tx`), and COMMIT
// only if both succeed. The magic: if any `?` returns early, `tx` goes out of
// scope uncommitted and Rust's Drop rolls it back automatically — no
// explicit rollback code, no silent data loss.
//
// The accounts table has `CHECK (credits >= 0)`, so trying to overdraw Alice
// fails the UPDATE and the whole transaction rolls back.
//
// Requires: Postgres running, DATABASE_URL set, migrations applied.
//
// Run with:
//   cargo run --bin 05_transactions

use db_foundations::connect;
use uuid::Uuid;

/// Transfer `amount` credits from one user to another — atomically.
///
/// The `WHERE credits >= $1` guard means the deduct UPDATE affects 0 rows if
/// Alice can't cover it. We check `rows_affected()` and bail before crediting
/// Bob, so the transaction rolls back via Drop.
async fn transfer_credits(
    pool: &sqlx::PgPool,
    from: Uuid,
    to: Uuid,
    amount: i64,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    // Deduct — but only if Alice has enough. 0 rows = insufficient funds.
    let debit = sqlx::query!(
        "UPDATE accounts SET credits = credits - $1 \
         WHERE user_id = $2 AND credits >= $1",
        amount,
        from
    )
    .execute(&mut *tx)
    .await?;

    if debit.rows_affected() == 0 {
        // Returning early drops `tx` → automatic ROLLBACK via Drop. Bob keeps
        // his credits; Alice keeps hers. Nothing partially applied.
        anyhow::bail!("insufficient funds");
    }

    // Credit Bob.
    sqlx::query!(
        "UPDATE accounts SET credits = credits + $1 WHERE user_id = $2",
        amount,
        to
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

async fn balance(pool: &sqlx::PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT credits FROM accounts WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(row.credits)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = connect().await?;

    // Seed two users + accounts. Alice has 100, Bob has 0.
    sqlx::query!("DELETE FROM accounts").execute(&pool).await?;
    sqlx::query!("DELETE FROM users WHERE email IN ($1, $2)", "alice@tx.example", "bob@tx.example")
        .execute(&pool)
        .await?;
    let alice = sqlx::query!(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        "Alice", "alice@tx.example"
    )
    .fetch_one(&pool)
    .await?;
    let bob = sqlx::query!(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        "Bob", "bob@tx.example"
    )
    .fetch_one(&pool)
    .await?;
    sqlx::query!("INSERT INTO accounts (user_id, credits) VALUES ($1, $2)", alice.id, 100i64)
        .execute(&pool)
        .await?;
    sqlx::query!("INSERT INTO accounts (user_id, credits) VALUES ($1, $2)", bob.id, 0i64)
        .execute(&pool)
        .await?;

    println!("Start:  Alice={}  Bob={}", balance(&pool, alice.id).await?, balance(&pool, bob.id).await?);

    // A valid transfer: 40 from Alice → Bob.
    transfer_credits(&pool, alice.id, bob.id, 40).await?;
    println!("After transferring 40:  Alice={}  Bob={}",
             balance(&pool, alice.id).await?, balance(&pool, bob.id).await?);

    // An invalid transfer: Alice only has 60, try to send 1000.
    match transfer_credits(&pool, alice.id, bob.id, 1000).await {
        Ok(_) => println!("ERROR: overdraw transfer unexpectedly succeeded"),
        Err(e) => {
            println!("Overdraw rejected (rolled back): {e}");
            println!("After failed transfer: Alice={}  Bob={}  ← unchanged",
                     balance(&pool, alice.id).await?, balance(&pool, bob.id).await?);
        }
    }

    // Cleanup.
    sqlx::query!("DELETE FROM accounts WHERE user_id IN ($1, $2)", alice.id, bob.id)
        .execute(&pool)
        .await?;
    sqlx::query!("DELETE FROM users WHERE id IN ($1, $2)", alice.id, bob.id)
        .execute(&pool)
        .await?;
    Ok(())
}