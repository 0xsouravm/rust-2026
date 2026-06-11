// Todo 1 — Hello, async
//
// Goal: understand that an `async fn` does nothing until awaited, and
// observe the difference between sequential `.await` and concurrent
// `tokio::join!`.
//
// Run with:
//   cargo run --bin todo1_hello_async

use std::time::Instant;
use tokio::time::{sleep, Duration};

/// Sleeps for 100 ms, then returns a greeting.
async fn greet(name: &str) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("Hello, {name}")
}

#[tokio::main]
async fn main() {
    // --- Sequential: each .await blocks the next from starting.
    let start = Instant::now();
    let a = greet("Alice").await;
    let b = greet("Bob").await;
    let c = greet("Carol").await;
    let seq_elapsed = start.elapsed();

    println!("[sequential] {a}");
    println!("[sequential] {b}");
    println!("[sequential] {c}");
    println!("[sequential] elapsed: {seq_elapsed:?}  (~300 ms expected)");

    // --- Concurrent: all three run on the same task, interleaved at
    // their .await points, so total time is limited by the slowest.
    let start = Instant::now();
    let (a, b, c) = tokio::join!(
        greet("Dave"),
        greet("Eve"),
        greet("Frank"),
    );
    let par_elapsed = start.elapsed();

    println!("[concurrent] {a}");
    println!("[concurrent] {b}");
    println!("[concurrent] {c}");
    println!("[concurrent] elapsed: {par_elapsed:?}  (~100 ms expected)");

    // Exercise for the student:
    //   * What happens if you remove the `.await` in the sequential
    //     version? (Hint: the compiler warns about an unused `Future`.)
    //   * Why is `concurrent` only ~100 ms and not faster?
}
