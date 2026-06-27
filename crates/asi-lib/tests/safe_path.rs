use asi_lib::safe_path::resolve_safe_path;

#[tokio::test]
async fn test_existing_file_in_project() {
    let path = resolve_safe_path("Cargo.toml").await.unwrap();
    assert!(path.ends_with("Cargo.toml"));
}

#[tokio::test]
async fn test_new_file_in_project() {
    let path = resolve_safe_path("target/test_new_file.txt").await.unwrap();
    assert!(path.ends_with("test_new_file.txt"));
}

#[tokio::test]
async fn test_path_traversal_blocked() {
    let result = resolve_safe_path("../etc/passwd").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Access denied"));
}
