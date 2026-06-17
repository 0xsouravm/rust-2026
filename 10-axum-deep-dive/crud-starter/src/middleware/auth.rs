// src/middleware/auth.rs — a toy API-key middleware.
//
// Not JWT, not OAuth — just an `X-API-Key` header check. The point of
// this file is the **selective application** pattern: the layer is
// added ONLY to the protected sub-router, not the public health
// endpoint. See `src/router.rs` to see how it's wired.
//
// The middleware can either:
//
//   1. Pass through:    `Ok(next.run(req).await)`
//   2. Short-circuit:   `Err(StatusCode::UNAUTHORIZED)`

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

const API_KEY: &str = "letmein";

pub async fn api_key_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let key = req
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if key != API_KEY {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}
