use axum::{
    Json, Router,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
};
use serde_json::Value;

use crate::agent::instructions::AGENT_INSTRUCTIONS;

/// GET /api/docs — returns the agent's full instructions as Markdown.
async fn get_docs() -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/markdown; charset=utf-8"),
    );

    let body = format!(
        "# ASI Agent Documentation\n\nVersion: {}\n\n{}",
        crate::agent::instructions::AGENT_INSTRUCTIONS_VERSION,
        AGENT_INSTRUCTIONS
    );

    (StatusCode::OK, headers, body)
}

/// GET /api/openapi.json — OpenAPI 3.0 specification.
async fn openapi_json() -> Json<Value> {
    Json(serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "ASI — AI Coding Assistant API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "REST API for the ASI multi-agent coding assistant platform."
        },
        "servers": [{ "url": "http://localhost:3000", "description": "Development" }],
        "paths": {
            "/api/health": { "get": { "summary": "Liveness check", "operationId": "health", "responses": { "200": { "description": "OK" } } } },
            "/api/ready": { "get": { "summary": "Readiness check (DB + AI provider)", "operationId": "ready", "responses": { "200": { "description": "Ready" }, "503": { "description": "Degraded" } } } },
            "/api/version": { "get": { "summary": "Version info", "operationId": "version", "responses": { "200": { "description": "OK" } } } },
            "/api/chat": {
                "post": {
                    "summary": "Send chat message (SSE streaming)",
                    "operationId": "chat",
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["messages"],
                            "properties": {
                                "messages": { "type": "array", "items": { "type": "object", "properties": { "role": { "type": "string", "enum": ["user","assistant","system"] }, "content": { "type": "string" } } } },
                                "agent": { "type": "string", "enum": ["code","review"] },
                                "session_id": { "type": "string" }
                            }
                        }}}
                    },
                    "responses": {
                        "200": { "description": "SSE event stream (text/event-stream)" },
                        "400": { "description": "RFC 7807 Problem Details" },
                        "403": { "description": "Prompt injection detected" },
                        "429": { "description": "Rate limited" }
                    },
                    "security": [{ "bearerAuth": [] }]
                }
            },
            "/api/model": {
                "get": { "summary": "List AI models", "operationId": "listModels", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] },
                "post": { "summary": "Switch model", "operationId": "switchModel", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] }
            },
            "/api/metrics": { "get": { "summary": "Prometheus metrics", "operationId": "metrics", "responses": { "200": { "description": "text/plain" } } } },
            "/api/stats": { "get": { "summary": "Aggregate stats", "operationId": "stats", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] } },
            "/api/flags": {
                "get": { "summary": "List feature flags", "operationId": "listFlags", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] },
                "post": { "summary": "Toggle flag", "operationId": "setFlag", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] }
            },
            "/api/sessions": {
                "get": { "summary": "List sessions", "operationId": "listSessions", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] },
                "post": { "summary": "Create session", "operationId": "createSession", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] }
            },
            "/api/tools": { "get": { "summary": "List agent tools", "operationId": "listTools", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] } },
            "/api/search": { "post": { "summary": "Search codebase", "operationId": "search", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] } },
            "/api/eval": { "post": { "summary": "Evaluate agent", "operationId": "eval", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] } },
            "/api/feedback": { "post": { "summary": "Submit feedback", "operationId": "feedback", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] } },
            "/api/evolve": { "post": { "summary": "Trigger self-evolution", "operationId": "evolve", "description": "Requires EVOLVE_SECRET", "responses": { "200": { "description": "OK" } }, "security": [{ "bearerAuth": [] }] } },
            "/api/openapi.json": { "get": { "summary": "This OpenAPI spec", "operationId": "openapi", "responses": { "200": { "description": "OK" } } } }
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": { "type": "http", "scheme": "bearer", "bearerFormat": "JWT", "description": "Clerk JWT (Authorization header or __session cookie)" }
            },
            "schemas": {
                "ProblemDetails": {
                    "type": "object",
                    "properties": {
                        "type": { "type": "string" },
                        "title": { "type": "string" },
                        "status_code": { "type": "integer" },
                        "detail": { "type": "string" },
                        "instance": { "type": "string" }
                    }
                }
            }
        }
    }))
}

pub fn routes() -> Router {
    Router::new()
        .route("/docs", get(get_docs))
        .route("/openapi.json", get(openapi_json))
}
