use crate::clerk;
use crate::types::AuthenticatedUser;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;

/// Axum middleware that verifies Clerk JWT and injects AuthenticatedUser.
/// Routes that need auth should wrap with this layer.
pub async fn require_auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let token = clerk::extract_jwt_from_request(&request).ok_or(StatusCode::UNAUTHORIZED)?;

    match clerk::verify_clerk_jwt(&token).await {
        Ok(user) => {
            request.extensions_mut().insert(Arc::new(user));
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
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
