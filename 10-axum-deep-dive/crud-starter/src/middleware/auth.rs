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
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{
    decode,
    DecodingKey,
    Validation
};
use serde::{Deserialize, Serialize};

// authentication using JWT middleware
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,    // subject
    pub exp: usize,     // expiry (unix timestamp seconds)
    pub role: String,   // custom validation claim
}

pub async fn jwt_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // extract the string value after "Bearer " from the Authorization header
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // fetch the signature verification secret
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secure-local-dev-key".to_string());
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    // decode signature and match validity window rules
    let data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // inject the parsed claims into Axum's request context extensions
    req.extensions_mut().insert(data.claims);
    Ok(next.run(req).await)
}

// const API_KEY: &str = "letmein";

// pub async fn api_key_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
//     let key = req
//         .headers()
//         .get("X-API-Key")
//         .and_then(|v| v.to_str().ok())
//         .ok_or(StatusCode::UNAUTHORIZED)?;

//     if key != API_KEY {
//         return Err(StatusCode::UNAUTHORIZED);
//     }

//     Ok(next.run(req).await)
// }
