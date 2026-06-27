use axum::{
    Json, Router,
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Mutex;

use asi_ai_sdk::provider::model_registry::{build_default_registry, ModelInfo, ModelRegistry};

// ---------------------------------------------------------------------------
// Global model registry (lazy, shared across requests)
// ---------------------------------------------------------------------------

fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&mut ModelRegistry) -> R,
{
    static REGISTRY: std::sync::LazyLock<Mutex<Option<ModelRegistry>>> =
        std::sync::LazyLock::new(|| Mutex::new(None));

    let mut guard = REGISTRY.lock().unwrap();
    if guard.is_none() {
        *guard = Some(build_default_registry());
    }
    f(guard.as_mut().unwrap())
}

/// Return a snapshot of the available models.
fn list_available() -> Vec<ModelInfo> {
    with_registry(|r| r.list_models())
}

/// Switch to the model at the given index.
fn switch(index: usize) -> Result<(), String> {
    with_registry(|r| r.switch_active(index))
}

// ---------------------------------------------------------------------------
// Request type
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SwitchModelRequest {
    /// Index into the model list (0-based).
    index: usize,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/model — list all available models.
async fn list_models() -> Json<Value> {
    let models = list_available();
    Json(serde_json::json!(models))
}

/// POST /api/model — switch the active model.
///
/// Body: `{ "index": 0 }`
async fn switch_model(
    Json(body): Json<SwitchModelRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match switch(body.index) {
        Ok(()) => {
            let models = list_available();
            let active = models.get(body.index);
            Ok(Json(serde_json::json!({
                "status": "switched",
                "active": active,
                "models": models,
            })))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new()
        .route("/model", get(list_models).post(switch_model))
}
