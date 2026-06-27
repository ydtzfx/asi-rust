use axum::{Json, Router, routing::get};
use serde_json::Value;

/// GET /api/health — simple liveness check.
async fn health() -> Json<Value> {
    Json(serde_json::json!({"status": "ok"}))
}

/// GET /api/ready — readiness check that probes the database.
async fn ready() -> Json<Value> {
    let db_ok = check_db().await;
    Json(serde_json::json!({
        "status": if db_ok { "ready" } else { "degraded" },
        "database": db_ok,
    }))
}

/// Attempt a trivial `SELECT 1` to verify the DB connection pool is healthy.
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

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
}
