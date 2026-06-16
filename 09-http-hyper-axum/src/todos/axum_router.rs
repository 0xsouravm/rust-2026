// Axum Router, HTTP methods, and JSON body extraction
//
// Concepts:
//   - `Router::new()` and `.route(path, method_router)`.
//   - Chaining methods on a single route: `get(h).post(h).delete(h)`.
//   - The `Json<T>` extractor — Content-Type + body parsing + deserialization.
//   - The `(StatusCode, Json<T>)` tuple return for created resources.
//   - Automatic behaviour: 404 for unknown paths, 405 for unknown methods.
//
// Run with:
//   cargo run --bin axum_router
//   curl http://localhost:3002/
//   curl -X POST -H "Content-Type: application/json" \
//        -d '{"name":"Neo","email":"neo@zion.io"}' \
//        http://localhost:3002/users
//   curl http://localhost:3002/users

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;

// ── Models ────────────────────────────────────────────────────────────
//
// Convention: separate "in" (request) and "out" (response) types so the
// two can evolve independently. `CreateUser` has no `id` because the
// server assigns one.

#[derive(Deserialize, Debug)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
}

// ── Handlers ──────────────────────────────────────────────────────────

// GET / — plain text, 200 OK.
async fn root() -> &'static str {
    "Axum router basics"
}

// GET /users — list. In a real app this would read from a database.
async fn list_users() -> Json<Vec<User>> {
    Json(vec![
        User { id: 1, name: "xyz".into(),     email: "xyz@google.io".into() },
        User { id: 2, name: "abc".into(), email: "abc@google.io".into() },
    ])
}

// POST /users — create. Returns (StatusCode::CREATED, Json<User>).
//
// The tuple return is idiomatic Axum: first element is the status, the
// rest is the response body. Axum serialises `User` to JSON and sets
// Content-Type automatically.
async fn create_user(
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 42, // would come from the database
        name: payload.name,
        email: payload.email,
    };
    (StatusCode::CREATED, Json(user))
}

// A handler that demonstrates the `impl IntoResponse` return position.
// The exact return type is hidden behind `impl Trait` — useful when
// the response shape varies but you don't want to spell it out.
async fn maybe_error() -> impl IntoResponse {
    // Both branches return the same shape: (StatusCode, &'static str).
    // That's the constraint `impl IntoResponse` requires.
    if true {
        (StatusCode::OK, "all good")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "boom")
    }
}

// ── Router ────────────────────────────────────────────────────────────

fn build_router() -> Router {
    Router::new()
        .route("/", get(root))
        // One path, multiple methods. Chaining `.get().post()` is the
        // idiomatic way to attach several handlers to the same route.
        .route("/users", get(list_users).post(create_user))
        .route("/maybe", get(maybe_error))
}

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:3002".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("istening on http://{addr}");
    println!();
    println!("Try:");
    println!("  curl http://{addr}/");
    println!("  curl http://{addr}/users");
    println!("  curl -X POST -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"name\":\"Morpheus\",\"email\":\"m@zion.io\"}}' \\");
    println!("       http://{addr}/users");
    println!();
    println!("Notice: GET /nope → 404, DELETE /users → 405. All automatic.");

    axum::serve(listener, build_router()).await.unwrap();
}
