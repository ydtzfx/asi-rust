use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub sub: String, // Clerk user ID
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub org_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authorization header or cookie")]
    MissingCredentials,
    #[error("Invalid or expired token: {0}")]
    InvalidToken(String),
    #[error("Failed to fetch JWKS: {0}")]
    JwksError(String),
}
