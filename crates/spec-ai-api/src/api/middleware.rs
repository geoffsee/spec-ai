/// API authentication and middleware
use crate::api::auth::AuthService;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

/// Extension to store authenticated user info in request
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub username: String,
}

/// Axum middleware function for bearer token authentication
///
/// This middleware:
/// 1. Checks if auth is enabled in the AuthService
/// 2. If disabled, allows all requests through
/// 3. If enabled, validates the Bearer token from Authorization header
/// 4. Adds AuthenticatedUser extension to request if valid
pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Response {
    // If auth is not enabled, allow all requests
    if !auth_service.is_enabled() {
        return next.run(request).await;
    }

    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let Some(auth_str) = auth_header else {
        return unauthorized_response("Missing Authorization header");
    };

    // Must be Bearer token
    let Some(token) = auth_str.strip_prefix("Bearer ") else {
        return unauthorized_response("Invalid Authorization header format. Expected: Bearer <token>");
    };

    // Validate token
    let Some(username) = auth_service.validate_token(token) else {
        return unauthorized_response("Invalid or expired token");
    };

    // Add authenticated user to request extensions
    request.extensions_mut().insert(AuthenticatedUser { username });

    next.run(request).await
}

/// Create an unauthorized response with JSON error body
fn unauthorized_response(message: &str) -> Response {
    let body = serde_json::json!({
        "error": message,
        "code": "unauthorized"
    });

    (
        StatusCode::UNAUTHORIZED,
        [(header::CONTENT_TYPE, "application/json")],
        Json(body),
    )
        .into_response()
}

/// Legacy API key authentication (kept for backward compatibility)
pub struct ApiKeyAuth {
    api_key: Option<String>,
}

impl ApiKeyAuth {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    /// Check if API key authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.api_key.is_some()
    }

    /// Validate an API key
    pub fn validate(&self, key: &str) -> bool {
        match &self.api_key {
            Some(expected) => expected == key,
            None => true, // No auth required if not configured
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_auth_disabled() {
        let auth = ApiKeyAuth::new(None);
        assert!(!auth.is_enabled());
        assert!(auth.validate("any_key"));
    }

    #[test]
    fn test_api_key_auth_enabled() {
        let auth = ApiKeyAuth::new(Some("secret123".to_string()));
        assert!(auth.is_enabled());
        assert!(auth.validate("secret123"));
        assert!(!auth.validate("wrong_key"));
    }

    #[test]
    fn test_api_key_validation() {
        let auth = ApiKeyAuth::new(Some("my-secret-key".to_string()));

        assert!(auth.validate("my-secret-key"));
        assert!(!auth.validate(""));
        assert!(!auth.validate("wrong"));
    }
}
