// 02_path_and_query — Type-safe path params and query strings
//
// What this demonstrates:
//   - `{id}` syntax for path parameters (Axum 0.8)
//   - Typed path extraction: `Path<u64>` auto-parses — bad input → 422
//   - Tuple path extraction for multiple params
//   - `Query<T>` extractor with `Option<T>` for optional fields
//   - `Json<T>` extractor for request bodies
//
// Run with:
//   cargo run --bin 02_path_and_query
//   curl http://127.0.0.1:3011/users/42
//   curl http://127.0.0.1:3011/users/abc          # 422 (type mismatch)
//   curl 'http://127.0.0.1:3011/search?q=rust&limit=3'
//   curl -X POST -H 'Content-Type: application/json' \
//        -d '{"name":"Neo","email":"n@z.io"}' \
//        http://127.0.0.1:3011/users
//   curl http://127.0.0.1:3011/users/7/posts/9

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;

// ── Handlers ───────────────────────────────────────────────────────────

// Single typed path param — auto-422 on bad parse
async fn get_user(Path(id): Path<u64>) -> String {
    format!("GET /users/{id}")
}

// Multiple path params as a tuple (ordered by appearance)
async fn get_user_post(Path((user_id, post_id)): Path<(u64, u64)>) -> String {
    format!("GET /users/{user_id}/posts/{post_id}")
}

// Query string deserialized into a struct — `Option<T>` = optional
#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
    limit: Option<u32>,
    sort: Option<String>,
}

async fn search(Query(p): Query<SearchParams>) -> String {
    format!(
        "GET /search — q={:?}, limit={:?}, sort={:?}",
        p.q, p.limit, p.sort
    )
}

// JSON body extractor — last in the parameter list (body-consuming)
#[derive(Deserialize)]
struct NewUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

async fn create_user(Json(body): Json<NewUser>) -> (StatusCode, Json<User>) {
    let user = User { id: 1, name: body.name, email: body.email };
    (StatusCode::CREATED, Json(user))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users/{id}",              get(get_user))
        .route("/users/{user_id}/posts/{post_id}", get(get_user_post))
        .route("/search",                  get(search))
        .route("/users",                   post(create_user));

    let addr: SocketAddr = "127.0.0.1:3011".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("02_path_and_query listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
