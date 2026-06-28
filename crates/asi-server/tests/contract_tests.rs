/// Contract tests — verify API response shapes match the OpenAPI spec.
use asi_server::router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;

fn app() -> axum::Router {
    router::build_test_router()
}

#[tokio::test]
async fn problem_details_contract() {
    let app = app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/chat")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"messages":[]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), 10_000)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // RFC 7807 required fields
    assert!(json.get("type").is_some(), "Missing 'type' field");
    assert!(json.get("title").is_some(), "Missing 'title' field");
    assert!(json.get("status_code").is_some(), "Missing 'status_code' field");
    assert_eq!(json["status_code"], 400);

    // 'type' should be a URI path
    assert!(
        json["type"].as_str().unwrap().starts_with("/problems/"),
        "'type' should start with /problems/"
    );
}

#[tokio::test]
async fn health_endpoint_contract() {
    let app = app();
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

#[tokio::test]
#[ignore = "requires external AI provider (Ollama/DeepSeek) running"]
async fn ready_endpoint_contract() {
    let app = app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), 10_000)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Should have status + database fields; ai_provider may be false in test env
    assert!(json.get("status").is_some(), "Missing status field");
    assert!(json.get("database").is_some(), "Missing database field");
    // status should be either "ready" or "degraded"
    let status = json["status"].as_str().unwrap();
    assert!(
        status == "ready" || status == "degraded",
        "Unexpected status: {}",
        status
    );
}
