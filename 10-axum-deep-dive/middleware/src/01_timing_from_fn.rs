// 01_timing_from_fn — Custom middleware with `middleware::from_fn`
//
// What this demonstrates:
//   - Writing middleware as a plain async function
//   - Two parameters: the request and `Next` (the rest of the chain)
//   - `next.run(req).await` is the "pass-through" point — code before it
//     is "inbound", code after it is "outbound"
//   - The Tower `Service` trait under the hood (poll_ready + call)
//
// Run with:
//   cargo run --bin 01_timing_from_fn
//   curl http://127.0.0.1:3020/slow
//   curl http://127.0.0.1:3020/fast

use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use std::{net::SocketAddr, time::Instant};
use tokio::net::TcpListener;

// Middleware = an async fn with (Request, Next) → Response
async fn timing_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let path  = req.uri().path().to_string();

    // ↓ Everything below runs DURING the handler
    let response = next.run(req).await;

    // ↑ Everything above runs AFTER the handler returns
    let elapsed_ms = start.elapsed().as_millis();
    println!("[timing] {path} → {} ({} ms)",
        response.status().as_u16(), elapsed_ms);

    response
}

async fn slow() -> &'static str {
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    "slow response"
}

async fn fast() -> &'static str {
    "fast response"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/slow", get(slow))
        .route("/fast", get(fast))
        .layer(middleware::from_fn(timing_middleware));

    let addr: SocketAddr = "127.0.0.1:3020".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("01_timing_from_fn listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
