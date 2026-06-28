use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::Value;

/// GET /api/health — simple liveness check (always returns ok if the process is alive).
async fn health() -> Json<Value> {
    Json(serde_json::json!({"status": "ok"}))
}

/// GET /api/ready — readiness check: DB + AI provider.
async fn ready() -> impl IntoResponse {
    let db_ok = check_db().await;
    let provider_ok = check_provider().await;

    let overall = db_ok && provider_ok;
    let status_code = if overall {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let body = serde_json::json!({
        "status": if overall { "ready" } else { "degraded" },
        "database": db_ok,
        "ai_provider": provider_ok,
    });

    (status_code, Json(body))
}

async fn check_db() -> bool {
    let pool = asi_db::get_db();
    let mut conn = match pool.try_acquire() {
        Some(c) => c,
        None => return false,
    };
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&mut *conn)
        .await
        .is_ok()
}

/// Check if the AI provider is reachable.
async fn check_provider() -> bool {
    // Use a simple TCP check to Ollama (or skip if not configured).
    if let Ok(base_url) = std::env::var("OLLAMA_BASE_URL") {
        let health_url = format!("{}/../", base_url.trim_end_matches("/v1"));
        if let Ok(resp) = reqwest::get(&health_url).await {
            return resp.status().is_success();
        }
    }
    // DeepSeek: try a lightweight API check
    if std::env::var("DEEPSEEK_API_KEY").is_ok() {
        if let Ok(resp) = reqwest::get("https://api.deepseek.com/v1/models").await {
            return resp.status().is_success();
        }
    }
    // No known provider configured — assume degraded.
    false
}

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
}
