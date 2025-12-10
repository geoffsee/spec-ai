//! Authentication module for the HTTP API
//!
//! Provides:
//! - User credential management (loaded from JSON file)
//! - Password verification using PBKDF2-HMAC-SHA256
//! - Bearer token generation and validation using HMAC-SHA256

use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::{hmac, pbkdf2, rand as ring_rand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

/// Number of PBKDF2 iterations for password hashing
const PBKDF2_ITERATIONS: u32 = 100_000;

/// Length of the salt for password hashing
const SALT_LENGTH: usize = 16;

/// Length of the derived key for password hashing
const CREDENTIAL_LENGTH: usize = 32;

/// Token validity duration default (24 hours in seconds)
const DEFAULT_TOKEN_EXPIRY_SECS: u64 = 86400;

/// A user credential stored in the credentials file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredential {
    /// Username for authentication
    pub username: String,
    /// PBKDF2-hashed password (base64 encoded: salt + derived_key)
    pub password_hash: String,
}

/// Token payload that gets signed
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenPayload {
    /// Username this token belongs to
    pub sub: String,
    /// Token issue timestamp (Unix epoch seconds)
    pub iat: u64,
    /// Token expiration timestamp (Unix epoch seconds)
    pub exp: u64,
    /// Unique token ID
    pub jti: String,
}

/// Authentication service that manages credentials and tokens
#[derive(Clone)]
pub struct AuthService {
    /// Map of username to credential
    credentials: Arc<HashMap<String, UserCredential>>,
    /// HMAC key for signing tokens
    signing_key: Arc<hmac::Key>,
    /// Token expiry duration in seconds
    token_expiry_secs: u64,
    /// Whether auth is enabled
    enabled: bool,
}

impl std::fmt::Debug for AuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthService")
            .field("credentials_count", &self.credentials.len())
            .field("token_expiry_secs", &self.token_expiry_secs)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl AuthService {
    /// Create a new AuthService
    ///
    /// # Arguments
    /// * `credentials_file` - Optional path to JSON file containing credentials
    /// * `token_secret` - Optional secret for signing tokens (random if not provided)
    /// * `token_expiry_secs` - Token expiry duration in seconds
    /// * `enabled` - Whether authentication is enabled
    pub fn new(
        credentials_file: Option<&Path>,
        token_secret: Option<&str>,
        token_expiry_secs: Option<u64>,
        enabled: bool,
    ) -> Result<Self> {
        // Load credentials if file is provided
        let credentials = if let Some(path) = credentials_file {
            Self::load_credentials(path)?
        } else {
            HashMap::new()
        };

        // Create signing key from provided secret or generate random
        let signing_key = if let Some(secret) = token_secret {
            hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes())
        } else {
            let rng = ring_rand::SystemRandom::new();
            hmac::Key::generate(hmac::HMAC_SHA256, &rng)
                .map_err(|_| anyhow::anyhow!("Failed to generate signing key"))?
        };

        Ok(Self {
            credentials: Arc::new(credentials),
            signing_key: Arc::new(signing_key),
            token_expiry_secs: token_expiry_secs.unwrap_or(DEFAULT_TOKEN_EXPIRY_SECS),
            enabled,
        })
    }

    /// Create a disabled AuthService (no authentication required)
    pub fn disabled() -> Self {
        Self {
            credentials: Arc::new(HashMap::new()),
            signing_key: Arc::new(
                hmac::Key::new(hmac::HMAC_SHA256, b"disabled-auth-not-used"),
            ),
            token_expiry_secs: DEFAULT_TOKEN_EXPIRY_SECS,
            enabled: false,
        }
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Load credentials from a JSON file
    fn load_credentials(path: &Path) -> Result<HashMap<String, UserCredential>> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read credentials file: {}", path.display()))?;

        let credentials: Vec<UserCredential> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse credentials file: {}", path.display()))?;

        let mut map = HashMap::new();
        for cred in credentials {
            map.insert(cred.username.clone(), cred);
        }

        tracing::info!("Loaded {} user credentials", map.len());
        Ok(map)
    }

    /// Verify a username/password combination
    pub fn verify_password(&self, username: &str, password: &str) -> bool {
        let Some(credential) = self.credentials.get(username) else {
            return false;
        };

        // Decode the stored hash (base64: salt + derived_key)
        let Ok(stored_bytes) = URL_SAFE_NO_PAD.decode(&credential.password_hash) else {
            tracing::warn!("Invalid base64 in password hash for user: {}", username);
            return false;
        };

        if stored_bytes.len() != SALT_LENGTH + CREDENTIAL_LENGTH {
            tracing::warn!("Invalid password hash length for user: {}", username);
            return false;
        }

        let (salt, stored_hash) = stored_bytes.split_at(SALT_LENGTH);

        // Verify the password using PBKDF2
        pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
            salt,
            password.as_bytes(),
            stored_hash,
        )
        .is_ok()
    }

    /// Generate a bearer token for a user
    pub fn generate_token(&self, username: &str) -> Result<String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("System time before Unix epoch")?
            .as_secs();

        let payload = TokenPayload {
            sub: username.to_string(),
            iat: now,
            exp: now + self.token_expiry_secs,
            jti: uuid::Uuid::new_v4().to_string(),
        };

        // Serialize payload to JSON
        let payload_json = serde_json::to_string(&payload)?;
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

        // Sign the payload
        let signature = hmac::sign(&self.signing_key, payload_b64.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.as_ref());

        // Token format: payload.signature (both base64 encoded)
        Ok(format!("{}.{}", payload_b64, signature_b64))
    }

    /// Validate a bearer token and return the username if valid
    pub fn validate_token(&self, token: &str) -> Option<String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 2 {
            return None;
        }

        let payload_b64 = parts[0];
        let signature_b64 = parts[1];

        // Verify signature
        let Ok(signature_bytes) = URL_SAFE_NO_PAD.decode(signature_b64) else {
            return None;
        };

        if hmac::verify(&self.signing_key, payload_b64.as_bytes(), &signature_bytes).is_err() {
            return None;
        }

        // Decode and validate payload
        let Ok(payload_json) = URL_SAFE_NO_PAD.decode(payload_b64) else {
            return None;
        };

        let Ok(payload): Result<TokenPayload, _> = serde_json::from_slice(&payload_json) else {
            return None;
        };

        // Check expiration
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();

        if now > payload.exp {
            return None;
        }

        Some(payload.sub)
    }

    /// Hash a password for storage
    /// Returns base64-encoded salt + derived_key
    pub fn hash_password(password: &str) -> Result<String> {
        let rng = ring_rand::SystemRandom::new();

        // Generate random salt
        let mut salt = [0u8; SALT_LENGTH];
        ring_rand::SecureRandom::fill(&rng, &mut salt)
            .map_err(|_| anyhow::anyhow!("Failed to generate salt"))?;

        // Derive key from password
        let mut derived_key = [0u8; CREDENTIAL_LENGTH];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
            &salt,
            password.as_bytes(),
            &mut derived_key,
        );

        // Combine salt + derived_key and encode
        let mut combined = Vec::with_capacity(SALT_LENGTH + CREDENTIAL_LENGTH);
        combined.extend_from_slice(&salt);
        combined.extend_from_slice(&derived_key);

        Ok(URL_SAFE_NO_PAD.encode(&combined))
    }
}

/// Request body for token generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
}

/// Response body for successful token generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Bearer token
    pub token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Seconds until token expires
    pub expires_in: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_password_hashing() {
        let password = "my_secret_password";
        let hash = AuthService::hash_password(password).unwrap();

        // Hash should be base64 encoded
        let decoded = URL_SAFE_NO_PAD.decode(&hash).unwrap();
        assert_eq!(decoded.len(), SALT_LENGTH + CREDENTIAL_LENGTH);
    }

    #[test]
    fn test_password_verification() {
        let password = "test_password_123";
        let hash = AuthService::hash_password(password).unwrap();

        // Create a credentials file
        let credentials = vec![UserCredential {
            username: "testuser".to_string(),
            password_hash: hash,
        }];

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", serde_json::to_string(&credentials).unwrap()).unwrap();

        let auth = AuthService::new(
            Some(file.path()),
            Some("test_secret"),
            Some(3600),
            true,
        ).unwrap();

        // Correct password should verify
        assert!(auth.verify_password("testuser", password));

        // Wrong password should fail
        assert!(!auth.verify_password("testuser", "wrong_password"));

        // Unknown user should fail
        assert!(!auth.verify_password("unknown", password));
    }

    #[test]
    fn test_token_generation_and_validation() {
        let auth = AuthService::new(None, Some("test_secret"), Some(3600), true).unwrap();

        let token = auth.generate_token("testuser").unwrap();

        // Token should validate and return correct username
        let username = auth.validate_token(&token);
        assert_eq!(username, Some("testuser".to_string()));

        // Invalid token should fail
        assert!(auth.validate_token("invalid.token").is_none());
        assert!(auth.validate_token("notavalidtoken").is_none());
    }

    #[test]
    fn test_expired_token() {
        // Create auth service with 0 second expiry
        let auth = AuthService::new(None, Some("test_secret"), Some(0), true).unwrap();

        let token = auth.generate_token("testuser").unwrap();

        // Wait more than 1 second so the token is definitely expired
        // (expiry is checked at second granularity)
        std::thread::sleep(std::time::Duration::from_millis(1100));

        assert!(auth.validate_token(&token).is_none());
    }

    #[test]
    fn test_disabled_auth() {
        let auth = AuthService::disabled();
        assert!(!auth.is_enabled());
    }

    #[test]
    fn test_token_tampering() {
        let auth = AuthService::new(None, Some("test_secret"), Some(3600), true).unwrap();

        let token = auth.generate_token("testuser").unwrap();
        let parts: Vec<&str> = token.split('.').collect();

        // Tamper with payload
        let tampered_payload = URL_SAFE_NO_PAD.encode(b"{\"sub\":\"admin\",\"iat\":0,\"exp\":9999999999,\"jti\":\"fake\"}");
        let tampered_token = format!("{}.{}", tampered_payload, parts[1]);

        assert!(auth.validate_token(&tampered_token).is_none());
    }
}
