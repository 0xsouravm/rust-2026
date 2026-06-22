// src/error.rs — the single place that owns HTTP status codes.
//
// `#[from] DbErr` makes `?` in handlers convert a SeaORM error into AppError
// automatically. `IntoResponse` peeks inside DbErr to turn a unique-constraint
// violation into 409 and (SeaORM's) RecordNotFound into 404; everything else
// is a 500 with a sanitized message (the raw error is logged, not leaked).

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)] // Conflict/Validation are part of the error model; wired up as the API grows
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] DbErr),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("validation error: {0}")]
    Validation(String),
}

impl AppError {
    /// Postgres reports a unique violation as "duplicate key value violates
    /// unique constraint". String-match so it works across `DbErr` variants.
    pub fn is_unique(e: &DbErr) -> bool {
        e.to_string().to_lowercase().contains("duplicate key")
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) if AppError::is_unique(e) => {
                (StatusCode::CONFLICT, "resource already exists".to_string())
            }
            AppError::Database(e) => {
                tracing::error!(error = ?e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string())
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}