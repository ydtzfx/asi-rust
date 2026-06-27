use asi_server::routes;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::Value;
use tower::ServiceExt;

/// Build a minimal router that nests the routes under `/api`,
/// matching the production layout in `router.rs`.
fn build_test_router() -> Router {
    let api_routes = Router::new()
        .merge(routes::health::routes())
        .merge(routes::flags::routes())
        .merge(routes::version::routes());

    Router::new().nest("/api", api_routes)
}

// ---------------------------------------------------------------------------
// test_health_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_endpoint() {
    let app = build_test_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 10_000)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

// ---------------------------------------------------------------------------
// test_version_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_version_endpoint() {
    let app = build_test_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 10_000)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "asi-server");
    assert!(json["version"].is_string());
    assert!(!json["version"].as_str().unwrap().is_empty());
    assert!(json["rustc"].is_string());
}

// ---------------------------------------------------------------------------
// test_flags_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_flags_endpoint() {
    let app = build_test_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/flags")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 10_000)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_object());

    // All expected flags are present
    let expected_flags = [
        "multi-agent",
        "prompt-injection-defense",
        "audit-logging",
        "session-persistence",
        "model-fallback",
        "user-feedback",
        "read-only-mode",
    ];
    for flag in &expected_flags {
        assert!(
            json.get(*flag).is_some(),
            "flag '{}' is missing from GET /api/flags",
            flag
        );
    }

    // prompt-injection-defense is on by default
    assert_eq!(json["prompt-injection-defense"], true);
    // multi-agent is off by default
    assert_eq!(json["multi-agent"], false);
}
