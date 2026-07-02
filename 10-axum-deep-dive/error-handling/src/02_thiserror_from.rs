// 02_thiserror_from — `thiserror` for typed errors + auto `From` impls
//
// What this demonstrates:
//   - `#[derive(thiserror::Error)]` to implement `Display` + `Error` in one line
//   - `#[error("...")]` for the per-variant message
//   - `#[from]` to auto-generate `From<source_type>` for `?`
//   - `#[error(transparent)]` to forward Display to the source verbatim
//
// Run with:
//   cargo run --bin 02_thiserror_from
//   curl -i http://127.0.0.1:3031/parse        # 422 — Validation
//   curl -i http://127.0.0.1:3031/divide/0     # 400 — BadRequest
//   curl -i http://127.0.0.1:3031/missing/1   # 404 — NotFound
//   curl -i http://127.0.0.1:3031/secret       # 500 — wrapped ParseInt

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::TcpListener;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("internal error")]
    Internal(#[from] anyhow::Error),  // catch-all (see example 03)
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(m)   => (StatusCode::NOT_FOUND,            m.to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST,          m),
            AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m),
            AppError::Internal(e)   => {
                // Never leak the internals to the client — log them, return generic
                eprintln!("[internal] {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// Handlers that exercise `?` and `From` 

async fn parse(Path(s): Path<String>) -> Result<String, AppError> {
    // ParseIntError → AppError::BadRequest via a manual `From` impl below
    let n: i64 = s.parse().map_err(|e: std::num::ParseIntError| AppError::BadRequest(e.to_string()))?;
    Ok(format!("parsed = {n}"))
}

async fn divide(Path(s): Path<String>) -> Result<String, AppError> {
    let n: i64 = s.parse().map_err(|e: std::num::ParseIntError| AppError::BadRequest(e.to_string()))?;
    if n == 0 { return Err(AppError::BadRequest("cannot divide by zero".into())); }
    Ok(format!("100 / {n} = {}", 100 / n))
}

async fn missing(Path(id): Path<u64>) -> Result<String, AppError> {
    Err(AppError::NotFound(format!("item {id} not found")))
}

async fn secret() -> Result<String, AppError> {
    // io::Error → anyhow::Error (via std::io::Error's blanket From?) — actually no.
    // The cleanest route is: map_err to anyhow, then to AppError via #[from].
    std::fs::read_to_string("does-not-exist")
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok("unreachable".into())
}

// Auto From impls — so the `?` operator does the right thing
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Internal(e.into()) }
}
impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self { AppError::BadRequest(e.to_string()) }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/parse",        get(parse))      // try /parse/42 or /parse/notanumber
        .route("/divide/{s}",   get(divide))
        .route("/missing/{id}", get(missing))
        .route("/secret",       get(secret));

    let addr: SocketAddr = "127.0.0.1:3031".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("02_thiserror_from listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
