use std::error::Error as StdError;
use std::fmt;

/// Error category for API response classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    User,      // 400 Bad Request
    Security,  // 403 Forbidden
    Retryable, // 503 Service Unavailable
    Fatal,     // 500 Internal Server Error
}

/// Base agent error with code and optional context.
#[derive(Debug)]
pub struct AgentError {
    code: String,
    message: String,
    context: Option<String>,
}

impl AgentError {
    pub fn new(code: &str, message: &str, context: Option<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            context,
        }
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl StdError for AgentError {}

/// User input error — recoverable, returns 400.
#[derive(Debug)]
pub struct UserError(AgentError);

impl UserError {
    pub fn new(message: &str, context: Option<String>) -> Self {
        Self(AgentError::new("USER_ERROR", message, context))
    }
    pub fn message(&self) -> &str {
        self.0.message()
    }
    pub fn code(&self) -> &str {
        self.0.code()
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl StdError for UserError {}

/// Retryable error — transient, carries optional retry delay in ms.
#[derive(Debug)]
pub struct RetryableError {
    inner: AgentError,
    retry_after_ms: Option<u64>,
}

impl RetryableError {
    pub fn new(message: &str, retry_after_ms: Option<u64>) -> Self {
        Self {
            inner: AgentError::new("RETRYABLE", message, None),
            retry_after_ms,
        }
    }
    pub fn message(&self) -> &str {
        self.inner.message()
    }
    pub fn code(&self) -> &str {
        self.inner.code()
    }
    pub fn retry_after_ms(&self) -> Option<u64> {
        self.retry_after_ms
    }
}

impl fmt::Display for RetryableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
impl StdError for RetryableError {}

/// Fatal error — do not retry, returns 500.
#[derive(Debug)]
pub struct FatalError(AgentError);

impl FatalError {
    pub fn new(message: &str, context: Option<String>) -> Self {
        Self(AgentError::new("FATAL", message, context))
    }
    pub fn message(&self) -> &str {
        self.0.message()
    }
    pub fn code(&self) -> &str {
        self.0.code()
    }
}

impl fmt::Display for FatalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl StdError for FatalError {}

/// Security boundary violation.
#[derive(Debug)]
pub struct SecurityError(AgentError);

impl SecurityError {
    pub fn new(message: &str, context: Option<String>) -> Self {
        Self(AgentError::new("SECURITY", message, context))
    }
    pub fn message(&self) -> &str {
        self.0.message()
    }
    pub fn code(&self) -> &str {
        self.0.code()
    }
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl StdError for SecurityError {}

/// Classify any error into a category + message for HTTP response.
pub fn classify_error(err: &(dyn StdError + 'static)) -> (ErrorCategory, String) {
    if let Some(e) = err.downcast_ref::<UserError>() {
        (ErrorCategory::User, e.message().to_string())
    } else if let Some(e) = err.downcast_ref::<SecurityError>() {
        (ErrorCategory::Security, e.message().to_string())
    } else if let Some(e) = err.downcast_ref::<RetryableError>() {
        (
            ErrorCategory::Retryable,
            format!("{} (retry in {:?}ms)", e.message(), e.retry_after_ms()),
        )
    } else if let Some(e) = err.downcast_ref::<FatalError>() {
        (ErrorCategory::Fatal, e.message().to_string())
    } else {
        (ErrorCategory::Fatal, format!("Internal error: {}", err))
    }
}
