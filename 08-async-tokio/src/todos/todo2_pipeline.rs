// Todo 2 — Sequential vs. concurrent pipeline
//
// Goal: measure the wall-clock cost of N independent I/O-bound awaits
// done one-after-the-other vs. all at once with `tokio::join!`, and
// compare against the `tokio::spawn` + collect pattern.
//
// Run with:
//   cargo run --bin todo2_pipeline

use std::time::Instant;
use tokio::time::{sleep, Duration};

async fn fetch_data(id: u64, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("data-{id}")
}

#[tokio::main]
async fn main() {
    let delays = [200u64, 150, 100, 300];

    // --- Sequential: 200 + 150 + 100 + 300 = 750 ms
    let start = Instant::now();
    let mut seq = Vec::new();
    for (i, d) in delays.iter().enumerate() {
        seq.push(fetch_data(i as u64, *d).await);
    }
    let seq_elapsed = start.elapsed();
    println!("[sequential]   results: {seq:?}");
    println!("[sequential]   elapsed: {seq_elapsed:?}  (~750 ms expected)");

    // --- Concurrent with tokio::join! — limited by the slowest (300 ms).
    let start = Instant::now();
    let par = tokio::join!(
        fetch_data(0, delays[0]),
        fetch_data(1, delays[1]),
        fetch_data(2, delays[2]),
        fetch_data(3, delays[3]),
    );
    let par_elapsed = start.elapsed();
    println!("[join!]        results: {par:?}");
    println!("[join!]        elapsed: {par_elapsed:?}  (~300 ms expected)");

    // --- Stretch: spawn + collect with a Vec<JoinHandle>.
    // Why might this finish slightly later than `join!`?
    //   * Each `tokio::spawn` allocates a new task on the runtime,
    //     which involves pushing it onto a worker's local queue.
    //   * The handles are then awaited in a `for` loop, so the main
    //     task parks/unparks once per handle instead of being driven
    //     through all four at once.
    let start = Instant::now();
    let handles: Vec<_> = (0..delays.len())
        .map(|i| tokio::spawn(fetch_data(i as u64, delays[i])))
        .collect();
    let mut spawned = Vec::new();
    for h in handles {
        spawned.push(h.await.expect("task panicked"));
    }
    let spawned_elapsed = start.elapsed();
    println!("[spawn+collect] results: {spawned:?}");
    println!("[spawn+collect] elapsed: {spawned_elapsed:?}");
    println!("  -> typically a hair later than `join!` due to per-task overhead");
}
