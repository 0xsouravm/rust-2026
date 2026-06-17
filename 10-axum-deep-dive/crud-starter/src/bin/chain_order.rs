// src/bin/chain_order.rs — visualising EXECUTION ORDER.
//
// Same app as `cargo run`, but the layers log a tag as they enter and
// leave, so the order is visible in real time.
//
// Run with:
//   cargo run --bin chain_order
//   RUST_LOG=info cargo run --bin chain_order
//   curl -i http://127.0.0.1:3011/api/v1/users/1
//   curl -X POST -H 'X-API-Key: letmein' -H 'Content-Type: application/json' \
//        -d '{"name":"X","email":"x@y"}' http://127.0.0.1:3011/api/v1/users

use std::net::SocketAddr;

use axum::{
    extract::{Path, Request},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use tokio::net::TcpListener;
use tower::Service;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

// ── Tagged logger — prints "IN" and "OUT" for a named layer ───────────
//
// We use this everywhere so the chain order is visible in the logs.
async fn tagged(name: &str, req: Request, next: Next) -> Response {
    tracing::info!("[{}] IN  → {}", name, req.uri().path());
    let response = next.run(req).await;
    tracing::info!("[{}] OUT ← {} ({})", name, response.status().as_u16(), name);
    response
}

macro_rules! make_tagged {
    ($name:ident, $tag:literal) => {
        async fn $name(req: Request, next: Next) -> Response {
            tagged($tag, req, next).await
        }
    };
}

make_tagged!(layer_a, "A:trace");
make_tagged!(layer_b, "B:cors");
make_tagged!(layer_c, "C:request_id");
make_tagged!(layer_d, "D:timing");
make_tagged!(layer_e, "E:auth");
make_tagged!(layer_f, "F:rate_limit");

// ── Fake data ──────────────────────────────────────────────────────────

async fn list_users() -> Json<serde_json::Value> {
    Json(json!([{ "id": 1, "name": "Neo" }, { "id": 2, "name": "Trinity" }]))
}

async fn get_user(Path(id): Path<u64>) -> Json<serde_json::Value> {
    Json(json!({ "id": id, "name": "Stub" }))
}

async fn create_user(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!({ "id": 99, "body": body })))
}

// ── A from-scratch Tower::Service + Tower::Layer demo ──────────────────
//
// Skim this. The point is to see that `axum::middleware::from_fn`
// is sugar for "implement Service, wrap with Layer". See
// `src/middleware/tower_service.rs` for the full implementation.

#[allow(dead_code)]
#[derive(Clone)]
struct HelloService;
#[allow(dead_code)]
impl Service<Request> for HelloService {
    type Response = Response;
    type Error = std::convert::Infallible;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, std::convert::Infallible>> + Send>>;
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, _req: Request) -> Self::Future {
        Box::pin(async {
            Ok(Response::new(axum::body::Body::from("hello from a hand-rolled Tower::Service")))
        })
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let app = Router::new()
        .route("/api/v1/users", get(list_users).post(create_user))
        .route("/api/v1/users/{id}", get(get_user))
        // INNERMOST FIRST — added first = closest to the handler
        .layer(middleware::from_fn(layer_f))
        .layer(middleware::from_fn(layer_e))
        .layer(middleware::from_fn(layer_d))
        .layer(middleware::from_fn(layer_c))
        .layer(middleware::from_fn(layer_b))
        // OUTERMOST LAST
        .layer(middleware::from_fn(layer_a))
        .layer(CorsLayer::new()
            .allow_origin(["http://localhost:3000".parse().unwrap()])
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]));

    let addr: SocketAddr = "127.0.0.1:3011".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("chain_order listening on http://{addr}");
    tracing::info!("expected log order per request: A → B → C → D → E → F → handler → F → E → D → C → B → A");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
