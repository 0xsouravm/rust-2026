// 04_nested_errors — Each layer of your app has its own error type
//
// What this demonstrates:
//   - A RepositoryError (talks about tables, columns, connections)
//   - A ServiceError    (talks about business rules: "user not found")
//   - An AppError       (talks about HTTP: status codes)
//   - `From<Repo> for Service` and `From<Service> for App` are the boundaries
//   - `#[error(transparent)]` for pass-through wrapping
//
// Run with:
//   cargo run --bin 04_nested_errors
//   curl -i http://127.0.0.1:3033/users/0      # user not found
//   curl -i http://127.0.0.1:3033/users/1      # conflict
//   curl -i http://127.0.0.1:3033/users/2      # persistence / 500
//   curl -i http://127.0.0.1:3033/users/3      # account locked

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

// ── Layer 1: repository ────────────────────────────────────────────────

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("database connection failed")]
    Connection(#[source] std::io::Error),

    #[error("row not found in `{table}`")]
    NotFound { table: String },

    #[error("unique constraint on `{column}`")]
    UniqueViolation { column: String },

    #[error(transparent)]
    Other(#[from] std::io::Error),
}

// ── Layer 2: service ───────────────────────────────────────────────────

#[derive(Debug, Error)]
enum ServiceError {
    #[error("user with id {0} not found")]
    UserNotFound(u64),

    #[error("user {0} already exists")]
    DuplicateUser(u64),

    #[error("user {0} is locked")]
    AccountLocked(u64),

    #[error("failed to load user data")]
    Persistence(#[source] RepositoryError),
}

impl From<RepositoryError> for ServiceError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFound { table } if table == "users" =>
                ServiceError::UserNotFound(0),
            RepositoryError::UniqueViolation { column } if column == "id" =>
                ServiceError::DuplicateUser(0),
            other => ServiceError::Persistence(other),
        }
    }
}

// ── Layer 3: HTTP / app ────────────────────────────────────────────────

#[derive(Debug, Error)]
enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Validation(String),

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl From<ServiceError> for AppError {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::UserNotFound(id)     => AppError::NotFound(format!("user {id}")),
            ServiceError::DuplicateUser(id)   => AppError::Conflict(format!("user {id} exists")),
            ServiceError::AccountLocked(id)    => AppError::Validation(format!("user {id} locked")),
            ServiceError::Persistence(src)     => AppError::Internal(anyhow::Error::new(src)),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(m)   => (StatusCode::NOT_FOUND,            m.clone()),
            AppError::Conflict(m)   => (StatusCode::CONFLICT,             m.clone()),
            AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
            AppError::Internal(_)   => (StatusCode::INTERNAL_SERVER_ERROR,
                                        "internal error".to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// ── Handlers ───────────────────────────────────────────────────────────

async fn get_user(Path(id): Path<u64>) -> Result<Json<serde_json::Value>, AppError> {
    let service: Result<(), ServiceError> = match id {
        0 => Err(RepositoryError::NotFound { table: "users".into() }.into()),
        1 => Err(RepositoryError::UniqueViolation { column: "id".into() }.into()),
        2 => Err(RepositoryError::Connection(
                   std::io::Error::new(std::io::ErrorKind::ConnectionReset, "db down")) .into()),
        3 => Err(ServiceError::AccountLocked(id)),
        _ => Ok(()),
    };
    service?;
    Ok(Json(json!({ "id": id })))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/users/{id}", get(get_user));

    let addr: SocketAddr = "127.0.0.1:3033".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("04_nested_errors listening on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
