// 02_tracing_layer — TraceLayer + tracing-subscriber for free observability
//
// What this demonstrates:
//   - Initializing tracing-subscriber with an EnvFilter
//   - Adding `TraceLayer::new_for_http()` to a router
//   - Reading log verbosity from the `RUST_LOG` env var
//
// Run with:
//   RUST_LOG=info cargo run --bin 02_tracing_layer
//   RUST_LOG=02_tracing_layer=debug,tower_http=info cargo run --bin 02_tracing_layer
//   curl http://127.0.0.1:3021/hello

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn hello() -> &'static str { "hello, world" }

#[tokio::main]
async fn main() {
    init_tracing();

    let app = Router::new()
        .route("/hello", get(hello))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = "127.0.0.1:3021".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("02_tracing_layer listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
