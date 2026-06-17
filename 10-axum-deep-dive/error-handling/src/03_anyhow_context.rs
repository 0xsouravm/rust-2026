// 03_anyhow_context — `anyhow` for application-level error chains
//
// What this demonstrates:
//   - `anyhow::Result<T>` shorthand for `Result<T, anyhow::Error>`
//   - `with_context(|| msg)` to attach a LAZY context message
//   - `bail!("msg")` to early-return with an ad-hoc error
//   - The full cause chain printed with `{:#}` (used in a fake "report" handler)
//
// Run with:
//   cargo run --bin 03_anyhow_context
//   curl -i http://127.0.0.1:3032/load/42
//   curl -i http://127.0.0.1:3032/load/0       # bail! path

use anyhow::{anyhow, Context, Result};
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

// Pretend data source that fails half the time
fn load_record(id: u64) -> Result<String> {
    if id == 0 {
        Err(anyhow!("id must be non-zero"))
    } else {
        // simulate a wrapped error
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "missing").into())
    }
}

async fn load(Path(id): Path<u64>) -> Result<Json<serde_json::Value>, AppError> {
    // The `?` here turns anyhow::Error into AppError via the explicit From impl
    let raw = load_record(id)
        .with_context(|| format!("loading record {id}"))?;

    if raw.is_empty() {
        return Err(anyhow!("record {id} had an empty body").into());
    }

    Ok(Json(json!({ "id": id, "raw": raw })))
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self { Self(e) }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { Self(e.into()) }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // The full chain is still here at the boundary — log it, return generic
        let report = format!("{:#}", self.0);
        eprintln!("[server] {report}");
        (StatusCode::INTERNAL_SERVER_ERROR,
         Json(json!({ "error": "internal error" }))).into_response()
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/load/{id}", get(load));

    let addr: SocketAddr = "127.0.0.1:3032".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("03_anyhow_context listening on http://{addr} — watch stderr for the chain");

    axum::serve(listener, app).await.unwrap();
}
