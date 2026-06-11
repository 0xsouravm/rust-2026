// Todo 10 — Concurrency vs. parallelism mini-experiment
//
// Goal: see that `tokio::spawn` on a single-threaded runtime is
// concurrent (interleaved) but NOT parallel, and that the wall-clock
// time is bounded by the slowest task, not the sum of all tasks.
//
// Run with:
//   cargo run --bin todo10_concurrency_vs_parallelism

use std::time::Instant;
use tokio::time::{sleep, Duration};

async fn slow_unit(id: u32) -> u32 {
    sleep(Duration::from_millis(500)).await;
    id
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Sequential: 4 × 500 ms = 2000 ms.
    let start = Instant::now();
    for i in 1..=4 {
        let v = slow_unit(i).await;
        println!("[seq] got {v} @ {:?}", start.elapsed());
    }
    println!("[seq] total: {:?}\n", start.elapsed());

    // Concurrent on a SINGLE-threaded runtime: still ~500 ms total
    // because the tasks are interleaved at their .await points — but
    // there is no true parallelism here. The 4 sleeps all complete
    // in the time of the slowest because sleep yields cooperatively.
    let start = Instant::now();
    let handles: Vec<_> = (1..=4).map(|i| tokio::spawn(slow_unit(i))).collect();
    for h in handles {
        let v = h.await.expect("task panicked");
        println!("[spawn] got {v} @ {:?}", start.elapsed());
    }
    println!("[spawn] total: {:?}", start.elapsed());
    // Comment in your own words:
    //   Why is this version ~500 ms and not 4× faster?
    //   * Concurrency ≠ parallelism.
    //   * Even with 4 spawned tasks, the runtime has only one worker
    //     thread, so the tasks take turns being polled. Each `sleep`
    //     yields to the executor, allowing the next task to make
    //     progress, but no two tasks are EVER running at the same
    //     instant. Total wall-clock is dominated by the longest
    //     single delay, not the sum — because none of the work is
    //     CPU-bound, just waiting.
    //   * To get true parallelism, switch to `flavor = "multi_thread"`
    //     and place CPU-bound work on the workers.
}
