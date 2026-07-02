// src/middleware/timing.rs — print how long each request takes.
//
// This is the canonical "from_fn" example. A middleware is just an
// async fn with the signature:
//
//     async fn (Request, Next) -> Response
//
// Code BEFORE `next.run(req).await` runs on the way IN.
// Code AFTER  `next.run(req).await` runs on the way OUT.
//
// This is also where `std::time::Instant` is most useful — start a
// timer, await the rest of the chain, log the elapsed time.

use std::time::Instant;

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

pub async fn timing_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path   = req.uri().path().to_string();

    // ↓ The handler + every middleware nested below runs here
    let response = next.run(req).await;

    // ↑ We're back. Log and return.
    let elapsed_ms = start.elapsed().as_millis();
    tracing::info!(
        method = %method,
        path   = %path,
        status = response.status().as_u16(),
        ms     = elapsed_ms as u64,
        "[timing]"
    );
    response
}
