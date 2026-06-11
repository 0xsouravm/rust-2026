// Todo 4 — spawn_blocking vs. blocking the executor
//
// Goal: see what happens when a CPU/blocking job runs directly inside
// an async task, then fix it with `tokio::task::spawn_blocking`.
//
// Run with:
//   cargo run --bin todo4_spawn_blocking

use std::time::Instant;
use tokio::time::{sleep, Duration};

/// A simulated 500 ms blocking job (sleeps the OS thread).
fn blocking_job(name: &str) -> String {
    std::thread::sleep(std::time::Duration::from_millis(500));
    format!("blocking_job({name}) done")
}

/// Bad: runs the blocking work on an async worker thread, starving every
/// other task that lands on the same worker for the full 500 ms.
async fn bad_blocking_job() -> String {
    // This is sync code, but we wrap it in an async fn to make the point.
    let result = blocking_job("bad");
    println!("  [bad] {result}");
    result
}

/// Good: hands the blocking work to a dedicated blocking-thread pool.
async fn good_blocking_job() -> String {
    let result = tokio::task::spawn_blocking(|| blocking_job("good"))
        .await
        .expect("blocking task panicked");
    println!("  [good] {result}");
    result
}

async fn short_task(id: u32) {
    sleep(Duration::from_millis(50)).await;
    println!("  short-{id} done @ {:?}", Instant::now());
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // We deliberately use a SINGLE worker thread so the starvation in
    // the BAD path is visible. With multi_thread the shorts would just
    // run on the other worker and you'd never see the problem.

    // ---------------- BAD PATH ----------------
    println!("--- BAD: blocking the executor (current_thread runtime) ---");
    let start = Instant::now();
    let bad_handle = tokio::spawn(bad_blocking_job());

    // While bad_blocking_job is occupying THE ONLY worker for 500 ms,
    // these 20 short tasks can't make progress. They all queue up and
    // start running only after the blocking job releases the thread.
    let shorts: Vec<_> = (1..=20)
        .map(|i| tokio::spawn(short_task(i)))
        .collect();
    for h in shorts {
        let _ = h.await;
    }
    let _ = bad_handle.await;
    println!("BAD total: {:?}\n", start.elapsed());
    // Expect: ~500 ms blocked first (all shorts parked), then 20 × 50 ms
    // of shorts firing in a tight burst. Total ≈ 1.5 s.

    // ---------------- GOOD PATH ----------------
    println!("--- GOOD: spawn_blocking (current_thread runtime) ---");
    let start = Instant::now();
    let good_handle = tokio::spawn(good_blocking_job());

    // The blocking job is moved off the async worker onto a dedicated
    // blocking-thread pool, so the single async worker is free to run
    // every short task. They each `sleep(50ms)` and interleave with
    // the blocking job; total is dominated by the 500 ms blocking job.
    let shorts: Vec<_> = (1..=20)
        .map(|i| tokio::spawn(short_task(i)))
        .collect();
    for h in shorts {
        let _ = h.await;
    }
    let _ = good_handle.await;
    println!("GOOD total: {:?}", start.elapsed());
    // Expect: ~500 ms total — all 20 short tasks fit comfortably while
    // the blocking job runs on the blocking pool.
}
