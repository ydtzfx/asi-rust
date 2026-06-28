use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// RFC 7807 Problem Details — standard error response format.
///
/// <https://datatracker.ietf.org/doc/html/rfc7807>
#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    /// HTTP status code.
    #[serde(skip)]
    pub status: StatusCode,

    /// A URI reference that identifies the problem type.
    #[serde(rename = "type")]
    pub ty: String,

    /// Short human-readable summary.
    pub title: String,

    /// HTTP status code as integer.
    pub status_code: u16,

    /// Human-readable explanation specific to this occurrence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// URI identifying the specific occurrence (for logging/tracing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl ProblemDetails {
    pub fn new(status: StatusCode, title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            status,
            ty: format!("/problems/{}", slugify(&title)),
            title,
            status_code: status.as_u16(),
            detail: None,
            instance: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    // ---- Convenience constructors ----

    pub fn bad_request(title: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, title)
    }

    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "Authentication required")
    }

    pub fn forbidden(title: &str) -> Self {
        Self::new(StatusCode::FORBIDDEN, title)
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "Resource not found")
    }

    pub fn too_many_requests(retry_after_secs: u64) -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded",
        )
        .with_detail(format!("Retry after {} seconds", retry_after_secs))
    }

    pub fn internal_error(detail: &str) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            .with_detail(detail.to_string())
    }

    pub fn service_unavailable(detail: &str) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, "Service unavailable")
            .with_detail(detail.to_string())
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = self.status;
        let mut response = Json(self).into_response();
        *response.status_mut() = status;
        response
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_details_serialization() {
        let pd = ProblemDetails::bad_request("Invalid input")
            .with_detail("The 'messages' field is required");
        let json = serde_json::to_string(&pd).unwrap();
        assert!(json.contains("Invalid input"));
        assert!(json.contains("messages"));
        assert!(json.contains("/problems/invalid-input"));
        assert_eq!(pd.status_code, 400);
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Rate limit exceeded"), "rate-limit-exceeded");
        assert_eq!(slugify("Hello World"), "hello-world");
    }
}
