use axum::{Json, Router, routing::get};
use serde_json::Value;

/// GET /api/stats — aggregate counts from the database.
async fn get_stats() -> Json<Value> {
    let pool = asi_db::get_db();

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let session_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let token_usage: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(token_used), 0) FROM sessions")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    Json(serde_json::json!({
        "users": user_count,
        "sessions": session_count,
        "tokens_used": token_usage,
        "projects": project_count,
        "audit_entries": audit_count,
    }))
}

pub fn routes() -> Router {
    Router::new().route("/stats", get(get_stats))
}
