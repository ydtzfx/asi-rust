use axum::{
    Router,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
};

use crate::agent::instructions::AGENT_INSTRUCTIONS;

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// GET /api/docs — returns the agent's full instructions as Markdown.
///
/// The response body is the system prompt / documentation that configures
/// agent behaviour, tool usage policies, output format, and constraints.
async fn get_docs() -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/markdown; charset=utf-8"),
    );

    let body = format!(
        "# ASI Agent Documentation\n\n\
         Version: {}\n\n{}",
        crate::agent::instructions::AGENT_INSTRUCTIONS_VERSION,
        AGENT_INSTRUCTIONS
    );

    (StatusCode::OK, headers, body)
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new().route("/docs", get(get_docs))
}
