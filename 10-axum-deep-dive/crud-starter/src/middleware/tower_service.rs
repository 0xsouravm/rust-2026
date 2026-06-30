use std::{future::Future, pin::Pin, task::{Context, Poll}, time::Instant};
use axum::extract::Request;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TimingService<S> {
    inner: S,
}

impl<S> Service<Request> for TimingService<S>
where
    S: Service<Request> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Response: 'static,
    S::Error: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start = Instant::now();
            let path = req.uri().path().to_string();
            let response = inner.call(req).await?;
            let ms = start.elapsed().as_millis();
            tracing::info!(path = %path, ms = %ms, "[timing]");
            Ok(response)
        })
    }
}

#[derive(Clone)]
pub struct TimingLayer;

impl<S> Layer<S> for TimingLayer {
    type Service = TimingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TimingService { inner }
    }
}

