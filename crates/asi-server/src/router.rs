use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::middleware::ResponseTimeLayer;
use crate::routes;

pub fn build_router(_leptos_options: leptos::config::LeptosOptions) -> Router {
    let api_routes = Router::new()
        .merge(routes::health::routes())
        .merge(routes::flags::routes())
        .merge(routes::sessions::routes())
        .merge(routes::chat::routes())
        .merge(routes::evolve::routes())
        .merge(routes::model::routes())
        .merge(routes::metrics::routes())
        .merge(routes::stats::routes())
        .merge(routes::version::routes())
        .merge(routes::feedback::routes())
        .merge(routes::search::routes())
        .merge(routes::tools::routes())
        .merge(routes::docs::routes())
        .merge(routes::eval::routes())
        .merge(routes::user::routes());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api", api_routes)
        .fallback(leptos_axum::render_app_to_stream(asi_app::App))
        .layer(ResponseTimeLayer)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
