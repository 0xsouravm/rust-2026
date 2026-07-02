// Todo 7 — Cancelling a spawned task
//
// Goal: see how `JoinHandle::abort()` cooperatively cancels a task at
// its next `.await` point, and that without `abort()` a task runs to
// completion.
//
// Run with:
//   cargo run --bin todo7_cancel_task

use std::time::Instant;
use tokio::time::{sleep, Duration};

async fn looped_task() {
    for i in 1..=10 {
        println!("  iteration {i} @ {:?}", Instant::now());
        sleep(Duration::from_millis(100)).await;
    }
    println!("  finished all 10 iterations");
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // ---------------- Cancelled ----------------
    println!("--- with abort() ---");
    let handle = tokio::spawn(looped_task());

    // Let it run for ~250 ms — about 2 iterations.
    sleep(Duration::from_millis(250)).await;
    handle.abort();

    match handle.await {
        Ok(_) => println!("task completed normally"),
        Err(e) if e.is_cancelled() => println!("✅ task was cancelled at its next .await"),
        Err(e) if e.is_panic() => eprintln!("task panicked: {e}"),
        Err(e) => eprintln!("other error: {e}"),
    }

    // ---------------- Not cancelled ----------------
    println!("\n--- without abort() ---");
    let start = Instant::now();
    let handle = tokio::spawn(looped_task());
    let _ = handle.await;
    println!("ran to completion in {:?}", start.elapsed());
    // ~1 s — all 10 iterations.
}
