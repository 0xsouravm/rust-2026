// src/middleware/request_id.rs — tags every request with a UUID so
// server logs and the client can correlate.
//
// Two ways to do this:
//
//   1. Hand-rolled `from_fn` middleware (this file)
//   2. `tower_http::request_id::SetRequestIdLayer` (used in router.rs)
//
// Hand-rolled is more code, but you can attach the id to whatever
// you like (logger context, response header, response body). The
// tower_http version is one line and does the header.

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub async fn request_id_middleware(req: Request, next: Next) -> Response {
    let id = req
        .headers()
        .get("X-Correlation-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    tracing::info!(request_id = %id, "incoming request");

    let mut response = next.run(req).await;
    if let Ok(hv) = HeaderValue::from_str(&id) {
        response.headers_mut().insert("X-Request-Id", hv);
    }
    response
}
