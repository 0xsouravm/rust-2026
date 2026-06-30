use crate::models::Claims;

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};



pub async fn jwt_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let key = DecodingKey::from_secret(secret.as_bytes());

    let validation = Validation::default();

    let data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(data.claims);

    Ok(next.run(req).await)
}