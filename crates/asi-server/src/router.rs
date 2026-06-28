use axum::Router;
use axum::middleware::from_fn;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::middleware::{GlobalRateLimitLayer, RequestIdLayer, ResponseTimeLayer};
use crate::routes;

pub fn build_router(_leptos_options: leptos::config::LeptosOptions) -> Router {
    let api_routes = build_api_routes(true);
    let cors = CorsLayer::permissive();

    Router::new()
        .nest("/api", api_routes)
        // Serve static frontend files from the static/ directory.
        // leptos SSR is disabled due to a leptos_meta 0.7.8 panic bug
        // ("you are using leptos_meta without a </head> tag").
        // The frontend is served as pre-built static HTML/CSS/JS.
        .fallback_service(ServeDir::new("static"))
        .layer(GlobalRateLimitLayer)
        .layer(RequestIdLayer)
        .layer(ResponseTimeLayer)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

pub fn build_test_router() -> Router {
    let api_routes = build_api_routes(false);
    Router::new().nest("/api", api_routes)
}

fn build_api_routes(require_auth: bool) -> Router {
    let public_routes = Router::new()
        .merge(routes::health::routes())
        .merge(routes::version::routes())
        .merge(routes::docs::routes());

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
