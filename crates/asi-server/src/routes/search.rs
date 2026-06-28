use axum::{Json, Router, extract::Query, http::StatusCode, routing::get};
use serde::Deserialize;
use serde_json::Value;

use asi_db::schema::Session;

// ---------------------------------------------------------------------------
// Request params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SearchParams {
    /// The search query string.
    q: String,
    /// Optional user-id filter.
    #[serde(default)]
    user_id: Option<String>,
    /// Maximum results (default 20, max 100).
    #[serde(default = "default_limit")]
    limit: i64,
    /// Pagination offset.
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    20
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// GET /api/search?q=...&user_id=...&limit=...&offset=...
///
/// Full-text search across session titles and context_json.
/// Uses SQLite `LIKE` for pattern matching (the `context_json` column stores
/// JSON conversation data that we search as raw text).
async fn search_sessions(
    Query(params): Query<SearchParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let query = params.q.trim();
    if query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Query parameter 'q' is required"})),
        ));
    }

    let pool = asi_db::get_db();
    // Escape SQL LIKE wildcards so `%` and `_` are matched literally.
    let escaped = query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{}%", escaped);

    // Search across title, context_json, and (if provided) user_id
    let results: Vec<Session> = if let Some(ref uid) = params.user_id {
        sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions
             WHERE user_id = ?
               AND (title LIKE ? OR context_json LIKE ?)
             ORDER BY updated_at DESC
             LIMIT ? OFFSET ?",
        )
        .bind(uid)
        .bind(&pattern)
        .bind(&pattern)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions
             WHERE title LIKE ? OR context_json LIKE ?
             ORDER BY updated_at DESC
             LIMIT ? OFFSET ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Search failed: {}", e)})),
        )
    })?;

    Ok(Json(serde_json::json!({
        "query": params.q,
        "count": results.len(),
        "results": results,
    })))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new().route("/search", get(search_sessions))
}
