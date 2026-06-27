use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{delete, get},
};
use serde::Deserialize;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CreateSessionRequest {
    #[serde(default = "default_user_id")]
    user_id: String,
    title: Option<String>,
}

fn default_user_id() -> String {
    "anonymous".to_string()
}

#[derive(Debug, Deserialize)]
struct ListSessionsParams {
    #[serde(default = "default_user_id")]
    user_id: String,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
struct DeleteSessionParams {
    #[serde(default = "default_user_id")]
    user_id: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/sessions?user_id=...&limit=...&offset=...
async fn list_sessions(
    Query(params): Query<ListSessionsParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = asi_db::get_db();
    asi_db::queries::sessions::list_sessions_by_user(pool, &params.user_id, params.limit, params.offset)
        .await
        .map(|sessions| Json(serde_json::json!(sessions)))
        .map_err(|e| internal_error(e.to_string()))
}

/// POST /api/sessions — create a new session.
/// Body: { "user_id": "...", "title": "..." }
async fn create_session(
    Json(body): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let pool = asi_db::get_db();
    asi_db::session_store::create_new_session(pool, &body.user_id, body.title.as_deref())
        .await
        .map(|s| (StatusCode::CREATED, Json(serde_json::json!(s))))
        .map_err(|e| internal_error(e.to_string()))
}

/// DELETE /api/sessions/:id?user_id=...
async fn delete_session(
    Path(id): Path<String>,
    Query(params): Query<DeleteSessionParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = asi_db::get_db();
    let rows = asi_db::queries::sessions::delete_session(pool, &id, &params.user_id)
        .await
        .map_err(|e| internal_error(e.to_string()))?;

    if rows > 0 {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Session not found"}))))
    }
}

/// GET /api/sessions/:id/export — export a session as JSON.
async fn export_session(
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = asi_db::get_db();
    match asi_db::session_store::get_session(pool, &id).await {
        Ok(Some(session)) => Ok(Json(serde_json::json!(session))),
        Ok(None) => Err(not_found("Session not found")),
        Err(e) => Err(internal_error(e.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn internal_error(msg: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": msg})),
    )
}

fn not_found(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": msg})))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new()
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/:id", delete(delete_session))
        .route("/sessions/:id/export", get(export_session))
}
