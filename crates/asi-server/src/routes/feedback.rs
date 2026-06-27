use axum::{Json, Router, http::StatusCode, routing::post};
use serde::Deserialize;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Request type
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FeedbackRequest {
    /// The feedback text or message from the user.
    message: String,
    /// Optional rating (e.g. "thumbs_up", "thumbs_down" or 1-5).
    #[serde(default)]
    rating: Option<String>,
    /// Optional user identifier.  Defaults to "anonymous".
    #[serde(default = "default_user")]
    user_id: String,
    /// Optional page or feature the feedback relates to.
    #[serde(default)]
    page: Option<String>,
    /// Optional session identifier for correlation.
    #[serde(default)]
    session_id: Option<String>,
}

fn default_user() -> String {
    "anonymous".to_string()
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// POST /api/feedback — persist user feedback.
///
/// Stores the feedback as an audit_log entry with action "feedback"
/// so it can be reviewed later alongside other audit events.
async fn submit_feedback(
    Json(body): Json<FeedbackRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.message.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Feedback message cannot be empty"})),
        ));
    }

    let pool = asi_db::get_db();

    let detail = serde_json::json!({
        "message": body.message,
        "rating": body.rating,
        "page": body.page,
    });

    asi_db::queries::audit::insert_audit_log(
        pool,
        &body.user_id,
        "feedback",
        &format!(
            "Feedback from {}: {}",
            body.user_id,
            truncate(&body.message, 200)
        ),
        Some(&detail.to_string()),
        body.session_id.as_deref(),
        None,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to store feedback: {}", e)})),
        )
    })?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// Truncate a string to at most `max` characters, appending "…" if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new().route("/feedback", post(submit_feedback))
}
