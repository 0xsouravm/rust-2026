use tokio::net::TcpStream;
use std::io;

// Async functions work perfectly with Result<T, E>
async fn connect_to_server(addr: &str) -> Result<TcpStream, io::Error> {
    let stream = TcpStream::connect(addr).await?;
    Ok(stream)
}

// Chaining async operations with ? — clean, readable, Rusty
async fn fetch_user_profile(
    db_url: &str,
    user_id: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    // Each of these could fail — ? propagates the error up
    let _stream = TcpStream::connect(db_url).await?;
    
    // Simulate query
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    Ok(format!("Profile for user {}", user_id))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // main can also return Result — runtime handles it gracefully
    match fetch_user_profile("127.0.0.1:5432", 42).await {
        Ok(profile) => println!("✅ {}", profile),
        Err(e) => eprintln!("❌ Error: {}", e),
    }
    Ok(())
}
