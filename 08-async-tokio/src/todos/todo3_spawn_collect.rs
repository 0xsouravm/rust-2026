// Todo 3 — Spawn and collect 10 tasks
//
// Goal: use `tokio::spawn` to launch 10 independent tasks, collect the
// `JoinHandle`s, await them, and properly handle all three `JoinError`
// cases (panic / cancelled / other).
//
// Run with:
//   cargo run --bin todo3_spawn_collect

use std::time::Instant;
use tokio::time::{sleep, Duration};

async fn worker(id: u32) -> String {
    sleep(Duration::from_millis(500)).await;
    format!("worker-{id} done")
}

#[tokio::main]
async fn main() {
    let start = Instant::now();

    // Spawn 10 tasks, collecting the JoinHandles.
    let handles: Vec<_> = (1..=10)
        .map(|i| tokio::spawn(worker(i)))
        .collect();

    // Await each handle, distinguishing the three failure modes.
    let mut completed = 0;
    for (i, h) in handles.into_iter().enumerate() {
        match h.await {
            Ok(result) => {
                completed += 1;
                println!("task {i}: ✅ {result}");
            }
            Err(e) if e.is_panic() => {
                eprintln!("task {i}: ❌ panicked: {e}");
            }
            Err(e) if e.is_cancelled() => {
                eprintln!("task {i}: ⚠️  cancelled: {e}");
            }
            Err(e) => {
                eprintln!("task {i}: ❓ other error: {e}");
            }
        }
    }

    println!("{completed}/10 tasks completed in {:?}", start.elapsed());
    // Total time should be ~500 ms, not 5 s — the tasks run concurrently.
    assert!(
        start.elapsed() < Duration::from_millis(900),
        "tasks should run concurrently"
    );
}
