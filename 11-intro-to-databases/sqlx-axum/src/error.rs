// src/error.rs — one AppError enum owns the status-to-JSON mapping. (DONE.)
//
// Handlers return `Result<T, AppError>` and the `?` operator "just works"
// because `#[from] sqlx::Error` generates the `From` impl. The single
// `IntoResponse` impl below is the only place that knows about HTTP status
// codes or the JSON error body — handlers never assemble an error response.
//
// This is provided so your handlers can use `?` from day one. Skim it, then
// move on — you shouldn't need to change it to complete the CRUD TODO.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("validation error: {0}")]
    Validation(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(ref db_err) => match db_err {
                sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "not found".into()),
                sqlx::Error::Database(e) if e.is_unique_violation() => {
                    (StatusCode::CONFLICT, "resource already exists".into())
                }
                other => {
                    tracing::error!(error = ?other, "database error");
                    (StatusCode::INTERNAL_SERVER_ERROR, "database error".into())
                }
            },
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}