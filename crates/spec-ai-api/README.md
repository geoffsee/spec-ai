# spec-ai-api

HTTP API server for the spec-ai framework.

## Overview

This crate provides a REST API and mesh server for remote agent interaction and coordination. It enables:

- **HTTP API**: RESTful endpoints for agent interaction
- **Mesh Server**: Distributed agent coordination and synchronization
- **Remote Access**: Network-based agent communication
- **Session Management**: Multi-session support with isolated contexts
- **Authentication**: Optional bearer token authentication

## Technology Stack

Built on modern async Rust web technologies:

- **axum** - Fast, ergonomic web framework
- **tower** - Modular service layers and middleware
- **tokio** - Async runtime
- **serde** - JSON serialization/deserialization
- **ring** - Cryptographic primitives for auth (PBKDF2, HMAC)

## Features

The API server provides:

- Agent chat endpoints
- Session management
- Tool execution via HTTP
- Real-time streaming responses
- Multi-agent coordination
- CORS support for web clients
- Optional bearer token authentication

## TLS (HTTPS)

The API server uses **mandatory TLS** for all connections. This provides:

- **Encryption**: All traffic is encrypted using TLS 1.2/1.3
- **Self-signed certificates**: Automatically generated if not provided
- **Certificate pinning**: Clients can verify the server by fingerprint

### Certificate Generation

On first startup, the server automatically generates a self-signed certificate and saves it to:
- Certificate: `~/.spec-ai/tls/server.crt`
- Private key: `~/.spec-ai/tls/server.key`

The certificate fingerprint is logged on startup and available via the `/cert` endpoint.

### Using Custom Certificates

To use your own certificate, configure in `spec-ai.config.toml` or pass to `ApiConfig`:

```toml
# In config (future support) or via ApiConfig builder:
# tls_cert_path = "/path/to/cert.pem"
# tls_key_path = "/path/to/key.pem"
```

### Certificate Verification (Swift/visionOS)

Since the server uses a self-signed certificate, clients should verify by fingerprint rather than chain of trust. The `/cert` endpoint returns certificate info:

```bash
curl -k https://localhost:3000/cert
```

Response:
```json
{
  "fingerprint": "AA:BB:CC:...",
  "certificate_pem": "-----BEGIN CERTIFICATE-----...",
  "not_after": "2025-12-10T...",
  "subject": "CN=spec-ai-server-localhost, O=spec-ai",
  "san": ["localhost", "127.0.0.1"]
}
```

In Swift, you can implement certificate pinning by comparing the fingerprint:

```swift
// Store the expected fingerprint (from /cert or server logs)
let expectedFingerprint = "AA:BB:CC:..."

// In URLSessionDelegate, verify the certificate
func urlSession(_ session: URLSession,
                didReceive challenge: URLAuthenticationChallenge,
                completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void) {
    guard let serverTrust = challenge.protectionSpace.serverTrust,
          let certificate = SecTrustGetCertificateAtIndex(serverTrust, 0) else {
        completionHandler(.cancelAuthenticationChallenge, nil)
        return
    }

    let fingerprint = sha256Fingerprint(of: certificate)
    if fingerprint == expectedFingerprint {
        completionHandler(.useCredential, URLCredential(trust: serverTrust))
    } else {
        completionHandler(.cancelAuthenticationChallenge, nil)
    }
}
```

## Authentication

The API supports optional bearer token authentication. When enabled, protected endpoints require a valid token.

### Quick Setup

Use the interactive setup script:

```bash
./scripts/setup-auth.sh
```

This will:
1. Create a credentials file with username/password entries
2. Generate secure PBKDF2 password hashes
3. Optionally update your config to enable auth

### Manual Setup

1. **Create a credentials file** (`~/.spec-ai/credentials.json`):

```json
[
  {"username": "admin", "password_hash": "BASE64_PBKDF2_HASH"},
  {"username": "user2", "password_hash": "BASE64_PBKDF2_HASH"}
]
```

2. **Generate password hashes** using the `/auth/hash` endpoint (requires auth to be enabled first with at least one user, or temporarily disabled):

```bash
curl -X POST http://localhost:3000/auth/hash \
  -H 'Content-Type: application/json' \
  -d '{"password": "your_secure_password"}'
```

3. **Enable authentication** in `spec-ai.config.toml`:

```toml
[auth]
enabled = true
credentials_file = "~/.spec-ai/credentials.json"
# token_expiry_secs = 86400  # 24 hours (default)
# token_secret = "your-secret-key"  # Optional: for token persistence across restarts
```

### API Endpoints

#### Public Endpoints (no auth required)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| GET | `/cert` | Get server certificate info and fingerprint |
| POST | `/auth/token` | Exchange username/password for bearer token |
| POST | `/auth/hash` | Generate password hash for credentials file |

#### Protected Endpoints (require auth when enabled)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/agents` | List available agents |
| POST | `/query` | Send query to agent |
| POST | `/stream` | Streaming query response (SSE) |
| GET/POST/PUT/DELETE | `/graph/*` | Knowledge graph operations |
| * | `/registry/*` | Mesh registry operations |
| * | `/messages/*` | Message routing |
| * | `/sync/*` | Graph synchronization |

### Authentication Flow

1. **Get a token**:
```bash
curl -X POST http://localhost:3000/auth/token \
  -H 'Content-Type: application/json' \
  -d '{"username": "admin", "password": "your_password"}'
```

Response:
```json
{
  "token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

2. **Use the token** on protected endpoints:
```bash
curl http://localhost:3000/agents \
  -H 'Authorization: Bearer eyJ...'
```

### Token Details

- Tokens are signed using HMAC-SHA256
- Default expiry: 24 hours (configurable)
- Tokens contain: username, issue time, expiry time, unique ID
- If `token_secret` is not set, a random key is generated at startup (tokens won't persist across restarts)

### Security Notes

- Password hashes use PBKDF2-HMAC-SHA256 with 100,000 iterations
- Credentials file should have restricted permissions (600)
- Consider using HTTPS in production (use a reverse proxy like nginx)
- Set `token_secret` in config for consistent token validation across restarts

## Dependencies

This crate depends on:
- `spec-ai-core` - Core agent runtime (with `api` feature enabled)
- `spec-ai-config` - Configuration management
- `spec-ai-policy` - Policy enforcement for API requests

## Usage

Enable the API server using the `api` feature flag:

```bash
cargo install spec-ai --features api
```

Start the server:

```bash
spec-ai server --port 3000
```

The API server is automatically started when configured in `spec-ai.config.toml`.

## Testing

Run API-specific tests:

```bash
cargo test -p spec-ai-api
```

For end-user documentation, see the main [spec-ai README](../../README.md).
