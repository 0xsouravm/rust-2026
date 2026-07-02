// Todo 5 — Async functions returning Result + `?`
//
// Goal: see that `?` works inside `async fn`, that `main` can return
// `Result`, and that errors propagate cleanly through the runtime.
//
// Run with:
//   cargo run --bin todo5_async_result

use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum AppError {
    Connection(String),
    Query(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Connection(s) => write!(f, "connection error: {s}"),
            AppError::Query(s) => write!(f, "query error: {s}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Connection(e.to_string())
    }
}

async fn fetch_user_profile(db_url: &str, user_id: u64) -> Result<String, AppError> {
    // `?` propagates the io::Error via the `From` impl above.
    let _stream = tokio::net::TcpStream::connect(db_url).await?;

    // Simulated query — 50 ms latency.
    sleep(Duration::from_millis(50)).await;

    if user_id == 0 {
        return Err(AppError::Query("user_id 0 is reserved".into()));
    }

    Ok(format!("Profile for user {user_id}"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- good address (will fail to connect — that's the point) ---");
    match fetch_user_profile("127.0.0.1:5432", 42).await {
        Ok(profile) => println!("✅ {profile}"),
        Err(e) => eprintln!("❌ {e}"),
    }

    println!("\n--- bad user_id (Query error path) ---");
    match fetch_user_profile("127.0.0.1:1", 0).await {
        Ok(profile) => println!("✅ {profile}"),
        Err(e) => eprintln!("❌ {e}"),
    }

    // If you wanted main to bubble errors instead of handling them:
    //   fetch_user_profile("127.0.0.1:5432", 42).await?;
    // The runtime will print the error and exit non-zero.
    Ok(())
}
