use axum::{Json, Router, routing::get};
use serde_json::Value;

/// GET /api/version — returns package version, server name, and rustc version.
async fn version() -> Json<Value> {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": "asi-server",
        "rustc": option_env!("RUSTC_VERSION").unwrap_or("unknown"),
    }))
}

pub fn routes() -> Router {
    Router::new().route("/version", get(version))
}
