// 03_request_id — Custom request-id middleware (one of the stretch goals)
//
// What this demonstrates:
//   - Generating a UUID per request
//   - Inserting it into request extensions (so handlers can read it)
//   - Echoing it back on the response via `X-Request-Id`
//   - Reading the request-id back from extensions in a handler
//
// Run with:
//   cargo run --bin 03_request_id
//   curl -i http://127.0.0.1:3022/whoami
//   curl -i -H 'X-Correlation-Id: my-trace-7' http://127.0.0.1:3022/whoami
//   # observe that the response header carries the same id

use axum::{
    extract::{Extension, Request},
    http::HeaderValue,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use uuid::Uuid;

// Newtype so we can use `Extension<RequestId>` without colliding with built-ins
#[derive(Clone)]
struct RequestId(String);

async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    // Honour a client-supplied correlation id; fall back to a fresh UUID
    let id = req
        .headers()
        .get("X-Correlation-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(RequestId(id.clone()));

    let mut response = next.run(req).await;

    // Add the id to the response so clients can quote it in support tickets
    if let Ok(hv) = HeaderValue::from_str(&id) {
        response.headers_mut().insert("X-Request-Id", hv);
    }
    response
}

async fn whoami(Extension(RequestId(id)): Extension<RequestId>) -> Json<serde_json::Value> {
    Json(json!({ "request_id": id }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/whoami", get(whoami))
        .layer(middleware::from_fn(request_id_middleware));

    let addr: SocketAddr = "127.0.0.1:3022".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("03_request_id listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
