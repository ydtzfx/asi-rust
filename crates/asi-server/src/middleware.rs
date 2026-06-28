use axum::http::Request;
use axum::response::Response;
use std::time::Instant;
use tower::{Layer, Service};

// ---------------------------------------------------------------------------
// Response-time middleware
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Request-ID middleware
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A Tower layer that adds `x-request-id` and `x-response-time` headers.
#[derive(Clone, Default)]
pub struct RequestIdLayer;

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for RequestIdService<S>
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
        let req_id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let start = Instant::now();
        let future = self.inner.call(req);
        Box::pin(async move {
            let mut response: Response = future.await?;
            let elapsed_ms = start.elapsed().as_millis();
            response.headers_mut().insert(
                "x-request-id",
                axum::http::HeaderValue::from_str(&format!("req_{}", req_id)).unwrap(),
            );
            response.headers_mut().insert(
                "x-response-time-ms",
                axum::http::HeaderValue::from_str(&elapsed_ms.to_string()).unwrap(),
            );
            response.headers_mut().insert(
                "x-api-version",
                axum::http::HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
            );
            Ok(response)
        })
    }
}

// ---------------------------------------------------------------------------
// Rate-limit middleware (global, per-endpoint)
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::Mutex;

/// Global rate-limiter shared across all requests.
/// Keys are endpoint prefixes; values track recent request timestamps.
static GLOBAL_LIMITER: std::sync::LazyLock<Mutex<HashMap<String, Vec<Instant>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

const GLOBAL_RATE_MAX: usize = 60; // max requests per window
const GLOBAL_RATE_WINDOW_SECS: u64 = 60; // window in seconds

/// A Tower layer that applies a global rate limit (60 req/min per endpoint).
#[derive(Clone, Default)]
pub struct GlobalRateLimitLayer;

impl<S> Layer<S> for GlobalRateLimitLayer {
    type Service = GlobalRateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GlobalRateLimitService { inner }
    }
}

#[derive(Clone)]
pub struct GlobalRateLimitService<S> {
    inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for GlobalRateLimitService<S>
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
        let path = req.uri().path().to_string();
        // Extract a coarse endpoint key (e.g. "/api/chat", "/api/sessions").
        let key = path
            .split('/')
            .take(3)
            .collect::<Vec<_>>()
            .join("/");

        let denied = {
            let mut map = GLOBAL_LIMITER.lock().unwrap();
            let now = Instant::now();
            let window = std::time::Duration::from_secs(GLOBAL_RATE_WINDOW_SECS);
            let timestamps = map.entry(key).or_default();

            // Evict expired entries.
            timestamps.retain(|t| now.duration_since(*t) < window);

            if timestamps.len() >= GLOBAL_RATE_MAX {
                true
            } else {
                timestamps.push(now);
                false
            }
        };

        if denied {
            let response = Response::builder()
                .status(axum::http::StatusCode::TOO_MANY_REQUESTS)
                .header("retry-after", "60")
                .body(axum::body::Body::from("Rate limit exceeded"))
                .unwrap();
            return Box::pin(async { Ok(response) });
        }

        let future = self.inner.call(req);
        Box::pin(async move { future.await })
    }
}
