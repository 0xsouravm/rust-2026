// src/main.rs — entry point for the CRUD starter.

mod error;
mod middleware;
mod models;
mod router;
mod routes;
mod state;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let app = router::build_app();

    let addr: SocketAddr = "127.0.0.1:3010".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("crud-starter listening on http://{addr}");

    // ConnectInfo so rate_limit_middleware can read the peer IP
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
