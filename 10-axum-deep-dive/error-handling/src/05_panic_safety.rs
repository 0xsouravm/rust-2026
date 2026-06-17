// 05_panic_safety — Don't let one bad request kill your service
//
// What this demonstrates:
//   - Installing a `panic::set_hook` BEFORE the server starts
//   - `tower_http::catch_panic::CatchPanicLayer::custom` to turn panics into 500s
//   - Why `catch_unwind` is wrong across `.await` (see slide notes)
//
// Run with:
//   cargo run --bin 05_panic_safety
//   curl -i http://127.0.0.1:3034/ok
//   curl -i http://127.0.0.1:3034/panic        # 500 — service stays alive
//   curl -i http://127.0.0.1:3034/ok           # still working

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::{any::Any, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::catch_panic::CatchPanicLayer;

// Custom 500 response for panics — log the message, return generic JSON
fn handle_panic(payload: Box<dyn Any + Send>) -> Response {
    let msg = payload
        .downcast_ref::<String>().cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown panic".to_string());

    eprintln!("[panic] handler panicked: {msg}");
    (StatusCode::INTERNAL_SERVER_ERROR,
     Json(json!({ "error": "internal error" }))).into_response()
}

async fn ok() -> &'static str { "ok" }

async fn panic_now() {
    // 1. unwrap on None — would normally crash the worker
    let v: Option<i32> = None;
    let _ = v.unwrap();
}

fn install_panic_hook() {
    // First line of main() in a real service
    std::panic::set_hook(Box::new(|info| {
        let location = info.location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".into());
        let message = info.payload()
            .downcast_ref::<String>().cloned()
            .or_else(|| info.payload().downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "non-string panic".into());
        eprintln!("PANIC at {location}: {message}");
    }));
}

#[tokio::main]
async fn main() {
    install_panic_hook();

    let app = Router::new()
        .route("/ok",     get(ok))
        .route("/panic",  get(panic_now))
        .layer(CatchPanicLayer::custom(handle_panic));

    let addr: SocketAddr = "127.0.0.1:3034".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("05_panic_safety listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
