// 01_apperror_into_response — The single most leveraged pattern in this deck
//
// What this demonstrates:
//   - Defining a custom `AppError` enum (no crate deps)
//   - Implementing `IntoResponse` for it: maps each variant to an HTTP status
//   - Returning `Result<T, AppError>` from a handler — `?` just works
//   - Uniform JSON error body: `{"error": "<message>"}`
//
// Run with:
//   cargo run --bin 01_apperror_into_response
//   curl -i http://127.0.0.1:3030/ok
//   curl -i http://127.0.0.1:3030/missing/42
//   curl -i http://127.0.0.1:3030/boom

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;

//  AppError 

#[derive(Debug)]
enum AppError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(m)   => (StatusCode::NOT_FOUND,            m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST,          m),
            AppError::Conflict(m)   => (StatusCode::CONFLICT,             m),
            AppError::Internal(m)   => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// Handlers 

async fn ok() -> &'static str { "ok" }

async fn fetch_one(Path(id): Path<u64>) -> Result<&'static str, AppError> {
    // The `?` operator only fires on the Err branch
    Err(AppError::NotFound(format!("item {id} not found")))
}

async fn boom() -> Result<&'static str, AppError> {
    Err(AppError::Internal("database is on fire".into()))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ok",           get(ok))
        .route("/missing/{id}", get(fetch_one))
        .route("/boom",         get(boom));

    let addr: SocketAddr = "127.0.0.1:3030".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("01_apperror_into_response listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
