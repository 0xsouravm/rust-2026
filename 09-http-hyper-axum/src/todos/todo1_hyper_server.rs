// todo1 — A simple HTTP server with raw Hyper 1.x
//
// Concepts:
//   - The Service trait and `service_fn` (turns an async fn into a Service).
//   - `Request<Incoming>` / `Response<Full<Bytes>>` as the core types.
//   - `TokioIo` adapter — bridges Tokio's AsyncRead/Write to Hyper's IO traits.
//   - Manual routing via `match` on (method, path).
//   - Returning `Result<Response<...>, Infallible>` — the handler can't fail.
//
// Run with:
//   cargo run --bin todo1_hyper_server
//   curl http://localhost:3000/
//   curl http://localhost:3000/health
//   curl -X POST -d "hello rust" http://localhost:3000/echo

use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

// Our handler: takes a Request, returns a Response.
// The error type is `Infallible` — the handler always produces a Response
// (even 404s come back as Ok(Response)).
async fn router(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    // Match on (method, path) — Rust's exhaustive pattern matching.
    match (req.method(), req.uri().path()) {
        // GET / — plain text welcome.
        (&Method::GET, "/") => Ok(Response::new(Full::new(Bytes::from(
            "Welcome to raw Hyper! 🦀",
        )))),

        // GET /health — JSON response. Content-Type must be set explicitly.
        (&Method::GET, "/health") => Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(r#"{"status":"ok"}"#)))
            .unwrap()),

        // POST /echo — collect the body, echo it back.
        // `req.collect()` reads the entire body stream into memory.
        (&Method::POST, "/echo") => {
            // collect() returns Collected<Bytes> — .to_bytes() gives us the bytes.
            let body_bytes = match req.collect().await {
                Ok(c) => c.to_bytes(),
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Full::new(Bytes::from("failed to read body")))
                        .unwrap());
                }
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .header("X-Echo-Size", body_bytes.len().to_string())
                .body(Full::new(body_bytes))
                .unwrap())
        }

        // Catch-all — 404.
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("404 Not Found")))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Hyper server listening on http://{addr}");

    loop {
        let (stream, _) = listener.accept().await?;
        // Wrap the raw TCP stream for Hyper.
        let io = TokioIo::new(stream);

        // Each connection gets its own Tokio task — thousands can run
        // concurrently without blocking each other.
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service_fn(router))
                .await
            {
                eprintln!("connection error: {e}");
            }
        });
    }
}
