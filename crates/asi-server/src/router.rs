use axum::Router;
use axum::body::Body;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::response::Response;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::middleware::{GlobalRateLimitLayer, ResponseTimeLayer};
use crate::routes;

/// Build the production router with auth on protected routes.
///
/// NOTE: leptos SSR (`render_app_to_stream`) is temporarily replaced with
/// a static handler due to a leptos_meta 0.7.8 panic ("you are using
/// leptos_meta without a </head> tag").  The Html/Body components in this
/// version do not emit the expected HTML skeleton.  Tracked for a follow-up
/// fix (requires either updating leptos_meta or providing a custom HTML shell).
pub fn build_router(_leptos_options: leptos::config::LeptosOptions) -> Router {
    let api_routes = build_api_routes(true);
    let cors = CorsLayer::permissive();

    Router::new()
        .nest("/api", api_routes)
        .fallback(fallback_handler)
        .layer(GlobalRateLimitLayer)
        .layer(ResponseTimeLayer)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Safe fallback — avoids leptos SSR panics that kill tokio workers.
async fn fallback_handler() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "text/plain")
        .body(Body::from("Not Found — API at /api"))
        .unwrap()
}

/// Build a router for integration tests (no auth middleware).
pub fn build_test_router() -> Router {
    let api_routes = build_api_routes(false);
    Router::new().nest("/api", api_routes)
}

fn build_api_routes(require_auth: bool) -> Router {
    // Public routes (no auth required).
    let public_routes = Router::new()
        .merge(routes::health::routes())
        .merge(routes::version::routes());

    // Protected routes.
    let protected = Router::new()
        .merge(routes::chat::routes())
        .merge(routes::flags::routes())
        .merge(routes::sessions::routes())
        .merge(routes::evolve::routes())
        .merge(routes::model::routes())
        .merge(routes::metrics::routes())
        .merge(routes::stats::routes())
        .merge(routes::feedback::routes())
        .merge(routes::search::routes())
        .merge(routes::tools::routes())
        .merge(routes::docs::routes())
        .merge(routes::eval::routes())
        .merge(routes::user::routes());

    let protected_routes = if require_auth {
        protected.layer(from_fn(asi_auth::middleware::require_auth))
    } else {
        protected
    };

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}
