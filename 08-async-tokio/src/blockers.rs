use std::thread;
use tokio::time::{sleep, Duration};

// ❌ WRONG — This blocks the entire executor thread!
async fn bad_crypto_work() {
    // This is pure CPU work that takes 500ms
    // While this runs, NO other tasks can make progress on this thread
    let _hash = sha256_of_huge_file(); // blocks for 500ms!
}

// ❌ ALSO WRONG — Never do blocking I/O in async code!
async fn bad_file_read() -> String {
    // std::fs::read blocks the OS thread — Tokio doesn't know about it
    thread::sleep(std::time::Duration::from_millis(500));
    println!("Finished sleeping, now reading file...");
    std::fs::read_to_string("assets/huge_file.txt").unwrap() // 😱
}

// ✅ CORRECT — Use spawn_blocking for CPU work or blocking I/O
async fn good_crypto_work() {
    println!("Starting expensive crypto work... this will not block the async executor!");
    let result = tokio::task::spawn_blocking(|| {
        // This runs on a SEPARATE, dedicated blocking thread pool
        // The async executor is FREE to run other tasks
        sha256_of_huge_file()
    })
    .await  // Wait for the blocking work to finish
    .expect("blocking task panicked");
    
    println!("Hash: {:?}", result);
}

// ✅ CORRECT — Use tokio::fs for async file I/O
async fn good_file_read() -> String {
    println!("Starting async file read... this will not block the async executor!");
    tokio::fs::read_to_string("assets/huge_file.txt")
        .await
        .unwrap()
}

fn sha256_of_huge_file() -> Vec<u8> {
    // Simulating expensive CPU work
    thread::sleep(std::time::Duration::from_millis(500));
    println!("Finished expensive crypto work!");
    vec![0u8; 32]
}


#[tokio::main()]
async fn main() {
    // // Bad ones first
    bad_crypto_work().await;
    let _ = bad_file_read().await;

    // // Good ones
    // good_crypto_work().await;
    // let _ = good_file_read().await;

    println!("All tasks completed!");

    // Dont forget using .await
    // 

}


// use std::sync::Mutex;
// use std::sync::Arc;

// // ❌ WRONG — Holding std::sync::Mutex across .await can deadlock!
// async fn bad_mutex_usage(shared: Arc<Mutex<Vec<u64>>>) {
//     let mut guard = shared.lock().unwrap(); // Lock acquired
    
//     do_some_io().await; // 😱 Guard held across await!
//     // If this task is rescheduled to a DIFFERENT thread,
//     // and another task on the ORIGINAL thread tries to lock — DEADLOCK
    
//     guard.push(42);
// } // Guard released here

// // ✅ CORRECT Option 1: Release the lock before .await
// async fn good_mutex_short(shared: Arc<Mutex<Vec<u64>>>) {
//     {
//         let mut guard = shared.lock().unwrap();
//         guard.push(42);
//     } // Guard released HERE — before the .await
    
//     do_some_io().await; // Safe! No lock held.
// }

// // ✅ CORRECT Option 2: Use tokio::sync::Mutex for async-aware locking
// async fn good_mutex_async(shared: Arc<tokio::sync::Mutex<Vec<u64>>>) {
//     let mut guard = shared.lock().await; // Async-aware!
//     do_some_io().await; // Safe — Tokio knows this task holds the lock
//     guard.push(42);
// }

// async fn do_some_io() { tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; }
