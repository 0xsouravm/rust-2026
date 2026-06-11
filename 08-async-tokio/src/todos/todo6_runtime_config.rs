// Todo 6 — Configuring the Tokio runtime
//
// Goal: build a runtime manually with `tokio::runtime::Builder`, then
// contrast it with `#[tokio::main(flavor = "current_thread")]`.
//
// Run with:
//   cargo run --bin todo6_runtime_config

use std::time::Instant;
use tokio::time::Duration;

async fn quick_task(id: u32) -> u32 {
    // CPU-bound busy work: 100 ms of `Instant::now()` polling so the
    // task does NOT yield. This is what makes the parallelism
    // difference visible between multi_thread and current_thread.
    let until = Instant::now() + Duration::from_millis(100);
    while Instant::now() < until {
        std::hint::black_box(Instant::now());
    }
    id
}

fn main() {
    // --- Multi-thread runtime, built by hand, with 2 worker threads. ---
    let start = Instant::now();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("manual-worker")
        .enable_all()
        .build()
        .expect("failed to build runtime");

    rt.block_on(async {
        let handles: Vec<_> = (1..=8).map(|i| tokio::spawn(quick_task(i))).collect();
        let mut sum = 0u32;
        for h in handles {
            sum += h.await.expect("task panicked");
        }
        println!("[multi-thread manual]  sum = {sum}, elapsed = {:?}", start.elapsed());
        // ~400 ms: 8 tasks × 100 ms of CPU / 2 workers = 4 × 100 ms.
    });
    rt.shutdown_background();

    // --- Current-thread runtime, also built by hand. ---
    let start = Instant::now();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");

    rt.block_on(async {
        let handles: Vec<_> = (1..=8).map(|i| tokio::spawn(quick_task(i))).collect();
        let mut sum = 0u32;
        for h in handles {
            sum += h.await.expect("task panicked");
        }
        println!("[current-thread manual] sum = {sum}, elapsed = {:?}", start.elapsed());
        // ~200 ms: one worker, but tasks yield at .await so the longest
        // single delay dominates. There is no parallelism here.
    });
}
