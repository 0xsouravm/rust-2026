// 05_cors_and_rate_limit — CORS layer + a simple in-memory rate limiter
//
// What this demonstrates:
//   - `tower_http::cors::CorsLayer` configured with explicit origins
//   - A token-bucket rate limiter (per client IP) implemented as middleware
//   - Ordering: TraceLayer outermost, CORS next, rate-limit closest to handler
//   - `from_fn_with_state` to pass shared state into a middleware
//
// Run with:
//   cargo run --bin 05_cors_and_rate_limit
//   # Hit it more than 5 times in 10s and you'll see 429 Too Many Requests
//   for i in $(seq 1 8); do curl -i http://127.0.0.1:3024/limited; done
//   # The free endpoint always works
//   curl -i http://127.0.0.1:3024/free
//   # CORS preflight (browser would do this before a real call)
//   curl -i -X OPTIONS -H 'Origin: http://localhost:3000' \
//        -H 'Access-Control-Request-Method: GET' http://127.0.0.1:3024/free

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

// ── Rate limiter ───────────────────────────────────────────────────────

#[derive(Debug)]
struct TokenBucket {
    tokens:     f64,
    capacity:   f64,
    refill_rate: f64,        // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self { tokens: capacity, capacity, refill_rate, last_refill: Instant::now() }
    }

    fn try_consume(&mut self) -> bool {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = Instant::now();
        if self.tokens >= 1.0 { self.tokens -= 1.0; true } else { false }
    }
}

type RateLimiterState = Arc<Mutex<HashMap<SocketAddr, TokenBucket>>>;

async fn rate_limit_middleware(
    State(state): State<RateLimiterState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Acquire-and-release the lock BEFORE awaiting — never hold std::sync::Mutex across .await
    let allowed = {
        let mut map = state.lock().unwrap();
        let bucket = map.entry(addr).or_insert_with(|| TokenBucket::new(5.0, 0.5));
        bucket.try_consume()
    };
    if allowed { Ok(next.run(req).await) } else { Err(StatusCode::TOO_MANY_REQUESTS) }
}

async fn free()        -> &'static str { "free endpoint — no rate limit" }
async fn limited()     -> &'static str { "limited endpoint — you got through!" }

#[tokio::main]
async fn main() {
    let rate_state: RateLimiterState = Arc::new(Mutex::new(HashMap::new()));

    let limited_routes = Router::new()
        .route("/limited", get(limited))
        .layer(middleware::from_fn_with_state(rate_state, rate_limit_middleware));

    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(std::time::Duration::from_secs(3600));

    let app = Router::new()
        .route("/free", get(free))
        .merge(limited_routes)
        // Order matters — CORS outermost so preflight OPTIONS short-circuits correctly
        .layer(cors);

    let addr: SocketAddr = "127.0.0.1:3024".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("05_cors_and_rate_limit listening on http://{addr}");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
