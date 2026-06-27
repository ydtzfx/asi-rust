use axum::http::Request;
use axum::response::Response;
use std::time::Instant;
use tower::{Layer, Service};

/// A Tower [`Layer`] that injects an `x-response-time` header into every response.
///
/// The value is the server-side processing time in milliseconds.
#[derive(Clone, Default)]
pub struct ResponseTimeLayer;

impl<S> Layer<S> for ResponseTimeLayer {
    type Service = ResponseTimeService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ResponseTimeService { inner }
    }
}

#[derive(Clone)]
pub struct ResponseTimeService<S> {
    inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for ResponseTimeService<S>
where
    S: Service<Request<ReqBody>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(req);
        Box::pin(async move {
            let mut response: Response = future.await?;
            let elapsed = start.elapsed().as_millis().to_string();
            response.headers_mut().insert(
                "x-response-time",
                axum::http::HeaderValue::from_str(&elapsed).unwrap(),
            );
            Ok(response)
        })
    }
}
