// src/middleware/rate_limit.rs — token-bucket rate limiter, applied
// SELECTIVELY (see router.rs) to write endpoints only.
//
// Algorithm:
//   - Each IP gets a bucket holding `capacity` tokens.
//   - `refill_rate` tokens drip back in per second.
//   - A request consumes 1 token. Empty bucket → 429.
//
// The state is shared via `State<RateLimiterState>` — that's the
// `from_fn_with_state` variant. The plain `from_fn` doesn't have
// state; `from_fn_with_state` does. See `bin/chain_order.rs` for a
// second example of this.

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
// import claims to see who the logged-in user is
use crate::middleware::auth::Claims;

#[derive(Debug)]
pub struct TokenBucket {
    pub tokens:      f64,   // current number of tokens in the bucket
    pub capacity:    f64,   // max tokens the bucket can hold(burst limit)
    pub refill_rate: f64,   // no. of tokens added back per second
    pub last_refill: Instant,   // exact time we last checked/refilled the bucket
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens:      capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self) -> bool {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = Instant::now();
        if self.tokens >= 1.0 { self.tokens -= 1.0; true } else { false }
    }
}

#[derive(Clone)]
pub struct RateLimiterState(pub Arc<Mutex<HashMap<String, TokenBucket>>>);

impl RateLimiterState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

pub async fn rate_limit_middleware(
    State(state): State<RateLimiterState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Apply the limiter to writes only. Reads are cheap and idempotent.
    if req.method() != axum::http::Method::POST {
        return Ok(next.run(req).await);
    }

    // look inside the request "extensions" to find the JWT claims
    // JWT authentication middleware must run before this for this to work
    let claims = req.
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user_id = claims.sub.clone();

    // find this user's bucket. If they don't have it, then create it.
    let allowed = {
        let mut map = state.0.lock().unwrap();
        let bucket = map
            .entry(user_id)
            .or_insert_with(|| TokenBucket::new(5.0, 0.5));
        bucket.try_consume()
    };

    if allowed {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}
