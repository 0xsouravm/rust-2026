use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::models::Claims;

#[derive(Debug)]
pub struct TokenBucket {
    pub tokens: f64,
    pub capacity: f64,
    pub refill_rate: f64,
    pub last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self) -> bool {
        let elapsed = self.last_refill.elapsed().as_secs_f64();

        self.tokens = (self.tokens + elapsed * self.refill_rate)
            .min(self.capacity);

        self.last_refill = Instant::now();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct RateLimiterState(
    pub Arc<Mutex<HashMap<String, TokenBucket>>>,
);

impl RateLimiterState {
    pub fn new() -> Self {
        Self(
            Arc::new(
                Mutex::new(HashMap::new()),
            ),
        )
    }
}

pub async fn rate_limit_middleware(
    State(state): State<RateLimiterState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if req.method() != axum::http::Method::POST {
        return Ok(next.run(req).await);
    }

    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let key = claims.sub.clone();

    let allowed = {
        let mut map = state.0.lock().unwrap();

        let bucket = map
            .entry(key)
            .or_insert_with(|| TokenBucket::new(5.0, 0.5));

        bucket.try_consume()
    };

    if allowed {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}