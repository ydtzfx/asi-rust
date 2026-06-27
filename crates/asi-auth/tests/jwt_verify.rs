use asi_auth::clerk::extract_jwt_from_request;

#[test]
fn test_extract_jwt_from_auth_header() {
    let req = axum::http::Request::builder()
        .uri("/api/chat")
        .header("authorization", "Bearer test.jwt.token")
        .body(())
        .unwrap();

    let token = extract_jwt_from_request(&req);
    assert_eq!(token, Some("test.jwt.token".to_string()));
}

#[test]
fn test_extract_jwt_from_cookie() {
    let req = axum::http::Request::builder()
        .uri("/api/chat")
        .header(
            "cookie",
            "other=val; __session=test.cookie.token; more=stuff",
        )
        .body(())
        .unwrap();

    let token = extract_jwt_from_request(&req);
    assert_eq!(token, Some("test.cookie.token".to_string()));
}

#[test]
fn test_no_auth_returns_none() {
    let req = axum::http::Request::builder()
        .uri("/api/chat")
        .body(())
        .unwrap();

    let token = extract_jwt_from_request(&req);
    assert_eq!(token, None);
}
