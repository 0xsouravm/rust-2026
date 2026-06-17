// src/middleware/logger.rs — visualising the EXECUTION ORDER of a
// request through the middleware chain.
//
// Every middleware that wraps a route runs like this:
//
//   ┌─ inbound code  ─┐
//   │                  │
//   │   next.run(req)  │   ← this is the "rest of the chain"
//   │                  │
//   └─ outbound code ──┘
//
// We log "IN" before calling next, and "OUT" after, so when you tail
// the server log you can see the exact order layers execute.

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

#[allow(dead_code)]
pub async fn logger_middleware(req: Request, next: Next) -> Response {
    let label = std::env::var("LOGGER_LABEL").unwrap_or_else(|_| "logger".to_string());
    tracing::info!("[{}] IN  → {}", label, req.uri().path());

    let response = next.run(req).await;

    tracing::info!("[{}] OUT ← {} ({})", label, response.status().as_u16(), label);
    response
}
