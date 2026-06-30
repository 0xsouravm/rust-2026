

use axum::{
    http::StatusCode,
    Json,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};

use crate::models::Claims;

pub async fn login(
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let username = body
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let password = body
        .get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if username.is_empty() || password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let secret =
        std::env::var("JWT_SECRET")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claims = Claims {
        sub: username.to_string(),
        role: "user".to_string(),
        exp: (
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600
        ) as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "token": token
    })))
}