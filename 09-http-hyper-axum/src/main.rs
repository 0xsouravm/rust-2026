// src/main.rs — Complete this!
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    age: u32,
}

async fn router(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    match (req.method(), req.uri().path()) {
        // TODO 1: GET /  → return "Hello, World! 🦀" with 200
        (&Method::GET, "/") => {
            let response = Full::new(Bytes::from("Hello, World! 🦀"));
            Ok(Response::new(response))
        },

        // TODO 2: GET /health → return JSON {"status":"ok","version":"1.0"}
        (&Method::GET, "/health") => {

            let version = "1.0";
            let body = json!({
                "status": "ok",
                "version": version
            }).to_string();
            
            Ok(
                Response::builder()
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(body)))
                .unwrap()
            )
        },

        // TODO 3: POST /echo → read body, echo it back, add X-Echo: true header
        // X- -> User / Server Defined HTTP Header (Custom)
        (&Method::POST, "/echo") => {
            let req_bytes = req.collect().await?.to_bytes();

            let user_data: User = serde_json::from_slice(&req_bytes)?;
            let modified_age = user_data.age + 10;

            let new_user = User {
                name: user_data.name,
                age: modified_age,
            };

            let new_response = serde_json::to_string(&new_user)?;

            Ok(
                Response::builder()
                .header("Content-Type", "text/plain")
                .header("X-Echo", "true")
                .body(Full::new(Bytes::from(new_response)))
                .unwrap()
            )
        },

        // TODO 4: GET /greet?name=Neo → read query param, return "Hello, Neo!"
        // HINT: req.uri().query() gives you "name=Neo"
        (&Method::GET, "/greet") => {

            let query = req.uri().query().unwrap_or(""); // name=Satyam
            let name = query.strip_prefix("name=").unwrap_or("World");

            let greeting = format!("Hello, {}!", name);

            let response = Full::new(Bytes::from(greeting));
            Ok(Response::new(response))
        },

        // Catch-all 404
        _ => {
            let body = Full::new(Bytes::from(r#"{"error":"not found"}"#));
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("🦀 Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service_fn(router))
                .await
            {
                eprintln!("Error: {}", e);
            }
        });
    }
}
