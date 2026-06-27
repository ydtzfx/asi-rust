use axum::{
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use asi_auth::middleware::{get_user_from_request, require_auth};
use asi_auth::types::AuthenticatedUser;

pub fn routes() -> Router {
    Router::new()
        .route("/user/me", get(user_me_handler))
        .route_layer(axum::middleware::from_fn(require_auth))
}

/// GET /api/user/me
///
/// Returns the authenticated user's information extracted from the Clerk
/// JWT cookie.  Requires the `require_auth` middleware to have run.
async fn user_me_handler(request: Request) -> impl IntoResponse {
    match get_user_from_request(&request) {
        Some(user) => {
            let user: &AuthenticatedUser = &*user;
            Json(serde_json::json!({
                "sub": user.sub,
                "email": user.email,
                "first_name": user.first_name,
                "last_name": user.last_name,
                "org_id": user.org_id,
            }))
            .into_response()
        }
        None => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "User context missing after authentication"
                })),
            )
                .into_response()
        }
    }
}
