// Building responses and sharing application state
//
// Concepts:
//   - Response shapes: `&'static str`, `(StatusCode, T)`, `Json<T>`, `Html<T>`,
//     and `Response::builder()` for full control.
//   - The `IntoResponse` trait: anything that implements it can be returned
//     from a handler. Strings, tuples, Json, Html all implement it.
//   - `State<AppState>` extractor: how handlers reach shared resources.
//   - The Arc<RwLock<T>> pattern for safe shared mutable state.
//
// Run with:
//   cargo run --bin responses_and_state
//   curl http://localhost:3003/plain
//   curl http://localhost:3003/created
//   curl http://localhost:3003/html
//   curl http://localhost:3003/counter
//   curl -i http://localhost:3003/missing  # demonstrates 404 with body

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;

// ── Application state ─────────────────────────────────────────────────
//
// All shared, mutable resources live behind a single struct. The struct
// derives `Clone` because Axum clones it for every request — and every
// clone of `Arc<T>` just bumps a refcount, not the underlying data.

#[derive(Clone)]
struct AppState {
    // A shared counter wrapped in Arc<RwLock<T>>.
    //   - Arc: shared ownership across async tasks.
    //   - RwLock: many readers OR one writer. Reads (counter reads in a
    //     list endpoint) don't block each other.
    request_count: Arc<RwLock<u64>>,

    // Pretend "database" — a HashMap<id, name>.
    items: Arc<RwLock<HashMap<u64, String>>>,
}

impl AppState {
    fn new() -> Self {
        let mut items = HashMap::new();
        items.insert(1, "red pill".to_string());
        items.insert(2, "blue pill".to_string());

        Self {
            request_count: Arc::new(RwLock::new(0)),
            items: Arc::new(RwLock::new(items)),
        }
    }
}

// ── Response building examples ────────────────────────────────────────

// 1. Plain string — auto 200 OK, text/plain.
async fn plain() -> &'static str {
    "just a string — Axum turns this into 200 OK with text/plain"
}

// 2. (StatusCode, &str) tuple — sets the status, body is the str.
async fn created() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "Resource created!")
}

// 3. JSON — most common response type. Use `serde_json::json!` for ad-hoc.
async fn quick_json() -> Json<serde_json::Value> {
    Json(json!({
        "service": "api",
        "status": "ok",
        "items": 2,
    }))
}

// 4. HTML — content type is text/html.
async fn html() -> Html<&'static str> {
    Html("<h1>Hello from Axum</h1><p>HTML is just a string with a different Content-Type.</p>")
}

// 5. Full control — Response::builder().
async fn with_headers() -> Response {
    Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Powered-By", "Rust")
        .body(Body::from(r#"{"queued":true}"#))
        .unwrap()
}

// 6. Custom response with HeaderMap via a tuple. Headers + status + body.
async fn with_header_map() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Request-Id",
        HeaderValue::from_static("req-abc-123"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );

    let body = Json(json!({ "trace": "abc-123" }));
    (StatusCode::OK, headers, body)
}

// ── State-using handlers ──────────────────────────────────────────────

// Increments a shared counter on every call. Notice `State<AppState>` —
// the extractor grabs the cloned state for this request.
async fn counter(State(state): State<AppState>) -> String {
    // Acquire a write lock — only one writer at a time, reads block.
    let mut count = state.request_count.write().unwrap();
    *count += 1;
    format!("This server has served {count} requests (this one included).")
}

// Reads from the shared HashMap. Many readers can hold the read lock
// concurrently — perfect for read-heavy endpoints.
async fn list_items(State(state): State<AppState>) -> Json<serde_json::Value> {
    let items = state.items.read().unwrap();
    Json(json!({
        "count": items.len(),
        "names": items.values().collect::<Vec<_>>(),
    }))
}

// ── Router ────────────────────────────────────────────────────────────

fn build_router() -> Router {
    Router::new()
        .route("/plain",      get(plain))
        .route("/created",    get(created))
        .route("/json",       get(quick_json))
        .route("/html",       get(html))
        .route("/headers",    get(with_headers))
        .route("/with-headers", get(with_header_map))
        .route("/counter",    get(counter))
        .route("/items",      get(list_items))
        // `.with_state` attaches the state — handlers that need it can
        // now extract `State<AppState>`.
        .with_state(AppState::new())
}

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:3003".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("listening on http://{addr}");
    println!();
    println!("Try:");
    println!("  curl http://{addr}/plain");
    println!("  curl http://{addr}/created");
    println!("  curl http://{addr}/json");
    println!("  curl http://{addr}/html");
    println!("  curl -i http://{addr}/headers");
    println!("  curl -i http://{addr}/with-headers");
    println!("  curl http://{addr}/counter    (try it a few times!)");
    println!("  curl http://{addr}/items");

    axum::serve(listener, build_router()).await.unwrap();
}
