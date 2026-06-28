use crate::clerk;
use crate::types::AuthenticatedUser;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;

/// Axum middleware that verifies Clerk JWT and injects AuthenticatedUser.
///
/// In production: requires a valid Clerk JWT (Authorization header or `__session` cookie).
///
/// In dev mode: when BOTH `CLERK_SECRET_KEY` starts with `sk_test_` AND
/// `ASI_DEV_AUTH_BYPASS=true` is set, the middleware accepts an `X-User-ID`
/// header as a fallback for local testing without a real Clerk instance.
/// **Never enable the bypass in production.**
pub async fn require_auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    // Try Clerk JWT verification first.
    if let Some(token) = clerk::extract_jwt_from_request(&request) {
        match clerk::verify_clerk_jwt(&token).await {
            Ok(user) => {
                request.extensions_mut().insert(Arc::new(user));
                return Ok(next.run(request).await);
            }
            Err(_) => {
                // JWT present but invalid — fall through to dev-mode check.
            }
        }
    }

    // Dev-mode fallback: requires BOTH a test Clerk key AND an explicit
    // opt-in.  A leaked test key alone is NOT sufficient to bypass auth.
    let dev_bypass_enabled = std::env::var("ASI_DEV_AUTH_BYPASS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let clerk_key = std::env::var("CLERK_SECRET_KEY").unwrap_or_default();
    if dev_bypass_enabled && clerk_key.starts_with("sk_test_") {
        if let Some(user_id) = request
            .headers()
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
        {
            let user = AuthenticatedUser {
                sub: user_id.to_string(),
                email: format!("{}@dev.local", user_id),
                first_name: Some("Dev".into()),
                last_name: Some("User".into()),
                org_id: None,
            };
            request.extensions_mut().insert(Arc::new(user));
            return Ok(next.run(request).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Extract the authenticated user from request extensions.
/// Returns None if the auth middleware hasn't run.
pub fn get_user_from_request<B>(
    request: &axum::http::Request<B>,
) -> Option<Arc<AuthenticatedUser>> {
    request
        .extensions()
        .get::<Arc<AuthenticatedUser>>()
        .cloned()
}
