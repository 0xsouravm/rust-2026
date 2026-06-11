// Todo 8 — Holding a Mutex across `.await` (the deadlock demo)
//
// Goal: see why `std::sync::Mutex` + `.await` is dangerous, and
// practice the two fixes — drop the guard before the `.await`, or
// switch to `tokio::sync::Mutex`.
//
// Run with:
//   cargo run --bin todo8_mutex_across_await
//
//     The BAD example is commented out. If you uncomment it, the
//     multi-thread runtime can deadlock. The comment block explains why.

use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

async fn do_some_io() {
    sleep(Duration::from_millis(10)).await;
}

// ❌ BAD — std::sync::Mutex guard held across an .await point.
//
// Why this can deadlock:
//   * You lock a std::sync::Mutex on worker thread W.
//   * You .await — Tokio is free to schedule another task onto W.
//   * That task also wants the same Mutex — and blocks (it can't yield,
//     because std::sync::Mutex is synchronous).
//   * Meanwhile your original task is parked on another worker holding
//     the lock. Result: nobody can make progress.
//
//   In a single-threaded current_thread runtime it's even sneakier —
//   the task holding the lock is parked and never re-polled, so
//   lock acquisition by anyone else deadlocks the entire runtime.
//
// async fn bad_mutex_usage(shared: Arc<StdMutex<Vec<u64>>>) {
//     let mut guard = shared.lock().unwrap();   // acquire
//     do_some_io().await;                       // 💥 guard held across .await
//     guard.push(42);                           // release (never reached if deadlocked)
// }

/// ✅ Option 1: drop the guard before the `.await`.
async fn good_mutex_short(shared: Arc<StdMutex<Vec<u64>>>) {
    {
        let mut guard = shared.lock().unwrap();
        guard.push(42);
    } // guard dropped HERE, before the .await
    do_some_io().await;
}

/// ✅ Option 2: use `tokio::sync::Mutex` — async-aware.
async fn good_mutex_async(shared: Arc<TokioMutex<Vec<u64>>>) {
    let mut guard = shared.lock().await;   // .await on acquisition
    do_some_io().await;
    guard.push(42);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let shared_std: Arc<StdMutex<Vec<u64>>> = Arc::new(StdMutex::new(Vec::new()));
    let shared_tok: Arc<TokioMutex<Vec<u64>>> = Arc::new(TokioMutex::new(Vec::new()));

    // Fix 1: scoped block.
    let s1 = Arc::clone(&shared_std);
    tokio::spawn(async move { good_mutex_short(s1).await });
    // Fix 2: tokio Mutex.
    let s2 = Arc::clone(&shared_tok);
    tokio::spawn(async move { good_mutex_async(s2).await });

    sleep(Duration::from_millis(50)).await;
    println!("std::sync::Mutex contents: {:?}", shared_std.lock().unwrap());
    println!("tokio::sync::Mutex contents: {:?}", shared_tok.lock().await);

    // Try uncommenting the bad path and see what happens:
    //   let s = Arc::clone(&shared_std);
    //   tokio::spawn(async move { bad_mutex_usage(s).await });
    // On a multi-core machine the runtime will likely wedge.
}
