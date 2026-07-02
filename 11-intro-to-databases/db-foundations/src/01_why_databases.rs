// 01_why_databases — Where data goes to live forever (or until DROP TABLE)
//
// Before we touch Postgres, let's feel *why* flat files are dangerous.
//
// Two tasks each read the same counter file, add 1, and write it back. There
// is no locking and no atomicity, so writes overlap and increments vanish —
// a textbook lost-update race. Run 50 concurrent increments and watch the
// file claim far fewer than 50.
//
// This is the problem ACID exists to solve:
//   A — Atomicity    all-or-nothing transactions (no half-applied writes)
//   C — Consistency  constraints keep state valid (CHECK, FK, UNIQUE, NOT NULL)
//   I — Isolation    concurrent transactions don't see each other's work-in-progress
//   D — Durability   once committed, data survives a crash — full stop
//
// No database is needed for this example — that's the point: a flat file is
// *not* a database.
//
// Run with:
//   cargo run --bin 01_why_databases

use std::fs;
use std::time::Duration;

const FILE: &str = "tmp/counter.json";

async fn increment_once() -> std::io::Result<()> {
    // READ: what's in the file right now?
    let n: u64 = fs::read_to_string(FILE)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    // ❌ Gap: another task can read the same `n`, both compute n+1, and one
    //    write clobbers the other. Sleep to widen the race window so we
    //    reliably lose updates even on a fast machine.
    tokio::time::sleep(Duration::from_millis(1)).await;

    // WRITE: our n+1 — overwrites whatever is there, even another task's write.
    fs::create_dir_all("tmp")?;
    fs::write(FILE, format!("{}", n + 1))?;
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = fs::remove_file(FILE);
    fs::create_dir_all("tmp")?;
    fs::write(FILE, "0")?;

    // 50 tasks, all racing on the same file.
    let mut tasks = Vec::new();
    for _ in 0..50 {
        tasks.push(tokio::spawn(increment_once()));
    }
    for t in tasks {
        let _ = t.await;
    }

    let final_count: u64 = fs::read_to_string(FILE)?.trim().parse().unwrap_or(0);
    println!("Asked for 50 concurrent increments.");
    println!("The file says: {final_count}");
    println!("Lost {} writes to a race condition — no Atomicity, no Isolation.",
             50 - final_count);
    println!();
    println!("A database fixes this with a transaction:");
    println!("  BEGIN");
    println!("    UPDATE accounts SET credits = credits - 100 WHERE user = 'alice';");
    println!("    UPDATE accounts SET credits = credits + 100 WHERE user = 'bob';");
    println!("  COMMIT   -- both, or neither. No lost updates, no half-transfers.");
    println!();
    println!("Next: cargo run --bin 02_sql_fundamentals  (real SQL against Postgres)");

    let _ = fs::remove_file(FILE);
    Ok(())
}