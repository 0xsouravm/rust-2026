// 03_nested_routers — Organizing routes by domain with .nest() and .merge()
//
// What this demonstrates:
//   - Building a per-domain `Router` function
//   - `.nest("/prefix", router)` adds a path prefix to every nested route
//   - `.merge(other)` combines routers at the SAME path level
//   - Auto 404 for paths that don't match any route
//
// Run with:
//   cargo run --bin 03_nested_routers
//   curl http://127.0.0.1:3012/health
//   curl http://127.0.0.1:3012/users
//   curl http://127.0.0.1:3012/products
//   curl http://127.0.0.1:3012/users/3
//   curl http://127.0.0.1:3012/api/v1/users     # nested-nested nesting

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

// ── Per-domain sub-routers ─────────────────────────────────────────────
// Each module returns a `Router` with paths RELATIVE to its future prefix.

fn users_router() -> Router {
    Router::new()
        .route("/",    get(list_users).post(create_user))
        .route("/{id}", get(get_user).delete(delete_user))
}

fn products_router() -> Router {
    Router::new()
        .route("/",     get(list_products))
        .route("/{id}", get(get_product))
}

// Re-export users at a versioned prefix — nested-nested nesting works too
fn api_v1() -> Router {
    Router::new().nest("/users", users_router())
}

async fn list_users()    -> &'static str { "GET /users — list" }
async fn create_user()   -> &'static str { "POST /users — create" }
async fn get_user()      -> &'static str { "GET /users/{id} — fetch" }
async fn delete_user()   -> &'static str { "DELETE /users/{id} — delete" }
async fn list_products() -> &'static str { "GET /products — list" }
async fn get_product()   -> &'static str { "GET /products/{id} — fetch" }
async fn health()        -> &'static str { "GET /health — ok" }

#[tokio::main]
async fn main() {
    // .nest() adds a prefix, .merge() joins at the same level
    let app = Router::new()
        .route("/health", get(health))
        .nest("/users",    users_router())
        .nest("/products", products_router())
        .nest("/api/v1",   api_v1());

    let addr: SocketAddr = "127.0.0.1:3012".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("03_nested_routers listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
