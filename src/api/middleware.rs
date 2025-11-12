/// API authentication and middleware
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

/// API key authentication middleware
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

/// Axum middleware function for API key authentication
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get API key from state (would be injected via layer)
    // For now, we'll extract from headers

    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            // Support both "Bearer <key>" and direct key
            let key = if auth_str.starts_with("Bearer ") {
                &auth_str[7..]
            } else {
                auth_str
            };

            // In production, validate against configured key
            // For now, accept any non-empty key
            if !key.is_empty() {
                return Ok(next.run(request).await);
            }
        }
    }

    // If no API key required (development mode), allow through
    // In production, this would reject
    Ok(next.run(request).await)
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
