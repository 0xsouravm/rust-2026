// src/error.rs — the AppError type.
//
// Step 4 of the hands-on.
//
// One error type for the whole service. Every handler returns
// `Result<T, AppError>` and `?` just works. The IntoResponse impl
// below does the HTTP status mapping AND the JSON shape — handlers
// never assemble an error response manually.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    // Future-proof: when you add a database, this is the variant
    // you'll use and `Internal` is the catch-all.
    #[allow(dead_code)]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(what)  => (StatusCode::NOT_FOUND,   what),
            AppError::BadRequest(why) => (StatusCode::BAD_REQUEST, why),
            AppError::Internal(why)   => (StatusCode::INTERNAL_SERVER_ERROR, why),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
