use std::time::Duration;

use crate::types::{AuthError, AuthenticatedUser};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use std::sync::RwLock;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClerkClaims {
    sub: String,
    sid: String,
    #[serde(rename = "__raw")]
    raw: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct JwkKey {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

/// Cache JWKS keys for 1 hour to avoid repeated network calls.
static JWKS_CACHE: RwLock<Option<(Vec<JwkKey>, std::time::Instant)>> = RwLock::new(None);

async fn fetch_jwks() -> Result<Vec<JwkKey>, AuthError> {
    // Check cache first
    {
        let cache = JWKS_CACHE.read().unwrap();
        if let Some((keys, cached_at)) = cache.as_ref()
            && cached_at.elapsed() < std::time::Duration::from_secs(3600)
        {
            return Ok(keys.clone());
        }
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| AuthError::JwksError(e.to_string()))?;
    let resp = client
        .get("https://api.clerk.com/v1/jwks")
        .send()
        .await
        .map_err(|e| AuthError::JwksError(e.to_string()))?;

    let jwks: JwksResponse = resp
        .json()
        .await
        .map_err(|e| AuthError::JwksError(e.to_string()))?;

    let keys = jwks.keys;
    let mut cache = JWKS_CACHE.write().unwrap();
    *cache = Some((keys.clone(), std::time::Instant::now()));

    Ok(keys)
}

/// Verify a Clerk session JWT and return the authenticated user.
/// The token can be the raw JWT from Clerk's `__session` cookie
/// or the standard Authorization header format.
pub async fn verify_clerk_jwt(token: &str) -> Result<AuthenticatedUser, AuthError> {
    let header = decode_header(token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    // Validate JWT type and algorithm before processing.
    if let Some(ref typ) = header.typ {
        if typ.to_uppercase() != "JWT" {
            return Err(AuthError::InvalidToken(format!(
                "Invalid token type: expected JWT, got {}",
                typ
            )));
        }
    }
    if header.alg != Algorithm::RS256 {
        return Err(AuthError::InvalidToken(format!(
            "Invalid algorithm: expected RS256, got {:?}",
            header.alg
        )));
    }

    let kid = header
        .kid
        .ok_or_else(|| AuthError::InvalidToken("Missing kid".into()))?;

    let jwks = fetch_jwks().await?;
    let jwk = jwks
        .iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| AuthError::InvalidToken(format!("Unknown kid: {}", kid)))?;

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://clerk.asi.com"]);
    validation.set_audience(&["asi-app"]);

    let token_data = decode::<ClerkClaims>(token, &decoding_key, &validation)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    // Extract user info from the raw claim if available
    let user = if let Some(raw) = token_data.claims.raw {
        serde_json::from_str::<AuthenticatedUser>(&raw).unwrap_or_else(|_| AuthenticatedUser {
            sub: token_data.claims.sub,
            email: String::new(),
            first_name: None,
            last_name: None,
            org_id: None,
        })
    } else {
        AuthenticatedUser {
            sub: token_data.claims.sub,
            email: String::new(),
            first_name: None,
            last_name: None,
            org_id: None,
        }
    };

    Ok(user)
}

/// Extract JWT from request headers or cookies.
pub fn extract_jwt_from_request<B>(request: &axum::http::Request<B>) -> Option<String> {
    // Try Authorization header first
    if let Some(auth) = request.headers().get("authorization")
        && let Ok(value) = auth.to_str()
        && let Some(token) = value.strip_prefix("Bearer ")
    {
        return Some(token.to_string());
    }

    // Try __session cookie (Clerk default)
    if let Some(cookie) = request.headers().get("cookie")
        && let Ok(cookie_str) = cookie.to_str()
    {
        for pair in cookie_str.split(';') {
            let pair = pair.trim();
            if let Some(value) = pair.strip_prefix("__session=") {
                return Some(value.to_string());
            }
        }
    }

    None
}
