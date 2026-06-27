use asi_lib::errors::*;

#[test]
fn test_user_error_creation() {
    let err = UserError::new("Invalid input", None);
    assert_eq!(err.message(), "Invalid input");
    assert_eq!(err.code(), "USER_ERROR");
}

#[test]
fn test_retryable_error_with_delay() {
    let err = RetryableError::new("Service unavailable", Some(5000));
    assert_eq!(err.retry_after_ms(), Some(5000));
}

#[test]
fn test_classify_user_error() {
    let err = UserError::new("Bad request", None);
    let (category, message) = classify_error(&err);
    assert_eq!(category, ErrorCategory::User);
    assert_eq!(message, "Bad request");
}

#[test]
fn test_classify_security_error() {
    let err = SecurityError::new("Path traversal attempt", None);
    let (category, message) = classify_error(&err);
    assert_eq!(category, ErrorCategory::Security);
    assert_eq!(message, "Path traversal attempt");
}

#[test]
fn test_fatal_error_not_retryable() {
    let err = FatalError::new("Panic: out of memory", None);
    assert!(!matches!(classify_error(&err).0, ErrorCategory::Retryable));
}
