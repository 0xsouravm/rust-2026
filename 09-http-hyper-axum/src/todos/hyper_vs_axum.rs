// Hyper vs Axum: same endpoints, two ways
//
// We expose three routes on TWO
// servers and you can flip a `const` to switch between them:
//
//   - `hyper_server()` — matches on (method, path), builds Response by hand.
//   - `axum_server()`  — uses Router::new().route(...)
//
// Both servers are real, runnable, and identical in behaviour. 
// Compare the line counts.
//
// Run with:
//   cargo run --bin hyper_vs_axum

use std::convert::Infallible;
use std::net::SocketAddr;

use axum::{routing::get, Json, Router};
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::{json, Value};
use tokio::net::TcpListener;

// ───────────────────────────────────────────────────────────── Hyper
//
// Every route is a match arm. Every Response is built by hand. Every
// status code is set explicitly. This is what Axum hides from you.

async fn hyper_handler(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Full::new(Bytes::from(
            "hello from hyper",
        )))),
        (&Method::GET, "/json") => Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(
                r#"{"engine":"hyper","logging":"verbose"}"#,
            )))
            .unwrap()),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
    }
}

async fn run_hyper_server(addr: SocketAddr) {
    let listener = TcpListener::bind(addr).await.expect("bind");
    println!("[hyper] listening on http://{addr}");

    loop {
        let (stream, _) = listener.accept().await.expect("accept");
        let io = TokioIo::new(stream);
        tokio::spawn(async move {
            let _ = http1::Builder::new()
                .serve_connection(io, service_fn(hyper_handler))
                .await;
        });
    }
}

// ───────────────────────────────────────────────────────────── Axum
//
// One line per route. Handlers are plain async fns. Return types tell
// Axum how to build the response. No match arms, no Response builders.

async fn axum_root() -> &'static str {
    "hello from axum"
}

async fn axum_json() -> Json<Value> {
    Json(json!({ "engine": "axum", "logging": "ergonomic" }))
}

async fn run_axum_server(addr: SocketAddr) {
    let app = Router::new()
        .route("/", get(axum_root))
        .route("/json", get(axum_json));

    let listener = TcpListener::bind(addr).await.expect("bind");
    println!("[axum]  listening on http://{addr}");

    axum::serve(listener, app).await.expect("serve");
}

// ───────────────────────────────────────────────────────────── Driver
//
// Flip MODE to switch between the two engines — same port, same routes,
// dramatically different code.

const MODE: &str = "axum"; // "hyper" or "axum"

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:3001".parse().unwrap();

    println!("Mode = {MODE}  (edit the const MODE in this file to switch)");
    println!();

    match MODE {
        "hyper" => run_hyper_server(addr).await,
        "axum" => run_axum_server(addr).await,
        other => panic!("unknown mode: {other}"),
    }
}
