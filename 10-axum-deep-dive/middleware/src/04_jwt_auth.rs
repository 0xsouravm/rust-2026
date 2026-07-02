// 04_jwt_auth — JWT auth middleware with the `jsonwebtoken` crate
//
// What this demonstrates:
//   - HS256 with a shared secret
//   - Extracting a Bearer token from the Authorization header
//   - Validating the token (signature + expiry) via `jsonwebtoken::decode`
//   - Injecting the decoded `Claims` into request extensions
//   - Public vs protected routes — apply the layer selectively
//
// Environment:
//   Set JWT_SECRET to anything non-empty, e.g.
//   $env:JWT_SECRET="dev-secret-please-change"      (PowerShell)
//   export JWT_SECRET=dev-secret-please-change      (bash)
//
// Run with:
//   cargo run --bin 04_jwt_auth
//   # 1. log in to get a token
//   curl -X POST -H 'Content-Type: application/json' \
//        -d '{"username":"neo","password":"matrix"}' \
//        http://127.0.0.1:3023/login
//   # 2. call the protected route with the token
//   TOKEN=...    (copy from step 1)
//   curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3023/me
//   curl http://127.0.0.1:3023/me                   # 401 — no token
//   curl -H "Authorization: Bearer bogus" http://127.0.0.1:3023/me  # 401

use axum::{
    extract::{Extension, Request},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,   // subject — usually user id
    exp: usize,    // expiry (unix seconds)
    role: String,  // custom claim
}

// Public — no auth
async fn login(Json(body): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, StatusCode> {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("");
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");

    // Toy check — in real code this hits your user store
    if username.is_empty() || password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let secret = std::env::var("JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let exp = (chrono_now_secs() + 3600) as usize;
    let claims = Claims { sub: username.into(), exp, role: "user".into() };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "token": token })))
}

// Protected — needs a valid JWT
async fn me(Extension(claims): Extension<Claims>) -> Json<serde_json::Value> {
    Json(json!({ "user_id": claims.sub, "role": claims.role }))
}

// The actual middleware
async fn jwt_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let secret = std::env::var("JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    let data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(data.claims);
    Ok(next.run(req).await)
}

// Tiny helper — `chrono` would be the usual choice; we avoid the dep here
fn chrono_now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[tokio::main]
async fn main() {
    // Layer is applied ONLY to the protected router
    let protected = Router::new()
        .route("/me", get(me))
        .layer(middleware::from_fn(jwt_middleware));

    let public = Router::new()
        .route("/login", post(login));

    let app = Router::new()
        .merge(public)
        .merge(protected);

    let addr: SocketAddr = "127.0.0.1:3023".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("04_jwt_auth listening on http://{addr} (set JWT_SECRET first)");

    axum::serve(listener, app).await.unwrap();
}
