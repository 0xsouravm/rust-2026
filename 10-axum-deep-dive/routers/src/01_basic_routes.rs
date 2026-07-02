// 01_basic_routes — HTTP method routing with auto-responses
//
// What this demonstrates:
//   - `Router::new()` and `.route(path, method_handler)`
//   - Chaining multiple methods on the same path: `.get().post().delete()`
//   - Auto-behaviors Axum gives you for free:
//       OPTIONS /items  → 200 OK with `Allow` header
//       PATCH  /items   → 405 Method Not Allowed
//       GET    /nope    → 404 Not Found
//
// Run with:
//   cargo run --bin 01_basic_routes
//   curl -i http://127.0.0.1:3010/items
//   curl -i -X POST http://127.0.0.1:3010/items
//   curl -i -X PATCH http://127.0.0.1:3010/items     # 405
//   curl -i -X OPTIONS http://127.0.0.1:3010/items   # 200 + Allow
//   curl -i http://127.0.0.1:3010/nope               # 404

#[allow(unused_imports)] // post/delete are used in method chains — Rust's unused-imports lint can't see that
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn list_items() -> &'static str {
    "GET /items — list all items"
}

async fn create_item() -> &'static str {
    "POST /items — create an item"
}

async fn delete_item() -> &'static str {
    "DELETE /items — delete all items"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/items",
        get(list_items)
            .post(create_item)
            .delete(delete_item),
    );

    let addr: SocketAddr = "127.0.0.1:3010".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("01_basic_routes listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
