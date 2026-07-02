// src/middleware/tower_service.rs — what `from_fn` is hiding.
//
// Axum middleware is a tower::Layer wrapping a tower::Service.
// This file is the un-sugared version of:
//
//     async fn my_mw(req: Request, next: Next) -> Response { ... }
//
// Read it once to understand the machinery. Then go back to
// `from_fn` and never write this by hand again XD.
//
// What you'll see:
//   - A `Service` impl that takes a request and returns a future
//   - A `Layer` impl that wraps one Service in another
//   - `poll_ready` — Tower's backpressure signal
//   - `call` — the actual work, returning a `Future`
//
// Run from `bin/chain_order.rs` to see it in action, or skim the
// comments and move on to `timing.rs`.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};
use tower::{Layer, Service};

use axum::{extract::Request, response::Response};

// Service
//
// A Service takes a Request and produces a Response asynchronously.
// `poll_ready` is the backpressure slot; for our purposes we are
// always ready.
#[derive(Clone)]
pub struct TimingService<S> {
    inner: S,
}

impl<S> Service<Request> for TimingService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    type Response = Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Just forward to the inner service.
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        // Tower Service::call is synchronous and returns a Future.
        // We have to clone the inner service because `call` takes `&mut self`.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start = Instant::now();
            let path  = req.uri().path().to_string();
            let resp  = inner.call(req).await.map_err(Into::into)?;
            let ms    = start.elapsed().as_millis();
            tracing::info!(path = %path, ms = %ms, "[tower-service] timing");
            Ok(resp)
        })
    }
}

// Layer
//
// A Layer wraps a Service. `layer(timing_layer)` is sugar for
// `service.layer(timing_layer)`.
#[derive(Clone)]
pub struct TimingLayer;

impl<S> Layer<S> for TimingLayer {
    type Service = TimingService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        TimingService { inner }
    }
}
