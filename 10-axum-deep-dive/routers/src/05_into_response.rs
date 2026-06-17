// 05_into_response — The IntoResponse trait in action
//
// What this demonstrates:
//   - Built-in IntoResponse impls: &str, String, StatusCode, Json, Html
//   - Tuple form: `(StatusCode, body)` and `(StatusCode, headers, body)`
//   - `Result<T, E>` where both T and E implement IntoResponse
//   - `impl IntoResponse` for return-type flexibility
//   - Custom headers via `axum::http::HeaderMap`
//
// Run with:
//   cargo run --bin 05_into_response
//   curl -i http://127.0.0.1:3014/str
//   curl -i http://127.0.0.1:3014/created
//   curl -i http://127.0.0.1:3014/json
//   curl -i http://127.0.0.1:3014/html
//   curl -i http://127.0.0.1:3014/headers
//   curl -i http://127.0.0.1:3014/optional/42
//   curl -i http://127.0.0.1:3014/optional/0       # 404 via Result

use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;

// 1. &str → 200 OK, text/plain
async fn as_str() -> &'static str { "Hello from &str" }

// 2. Tuple (StatusCode, body) — sets a custom status
async fn as_created() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "resource created")
}

// 3. Json<T> — sets Content-Type: application/json automatically
async fn as_json() -> impl IntoResponse {
    Json(json!({ "id": 1, "name": "Neo" }))
}

// 4. Html — sets Content-Type: text/html
async fn as_html() -> Html<&'static str> {
    Html("<h1>Hello from Axum!</h1>")
}

// 5. Custom headers via tuple form
async fn with_headers() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("X-Request-Id", "demo-123".parse().unwrap());
    (StatusCode::OK, headers, Json(json!({ "ok": true })))
}

// 6. Result<T, E> — both sides implement IntoResponse
//    This is the keystone pattern that we will scale with AppError
//    in the error-handling examples.
async fn maybe_user(axum::extract::Path(id): axum::extract::Path<u64>)
    -> Result<Json<serde_json::Value>, StatusCode>
{
    if id == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(Json(json!({ "id": id, "name": "Trinity" })))
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/str",             get(as_str))
        .route("/created",        get(as_created))
        .route("/json",           get(as_json))
        .route("/html",           get(as_html))
        .route("/headers",        get(with_headers))
        .route("/optional/{id}",  get(maybe_user));

    let addr: SocketAddr = "127.0.0.1:3014".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("05_into_response listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
