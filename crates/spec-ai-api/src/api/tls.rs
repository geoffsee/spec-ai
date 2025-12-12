//! TLS certificate generation and management
//!
//! Provides self-signed certificate generation using rcgen for the API server.
//! The self-signed nature of the certificate can be used by clients (like the
//! visionOS app) to verify they're connecting to a legitimate spec-ai server
//! by validating the certificate fingerprint.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rcgen::{
    CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, SanType,
};
use ring::digest::{digest, SHA256};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Default certificate validity period (365 days)
const DEFAULT_CERT_VALIDITY_DAYS: u32 = 365;

/// Organization name for the certificate
const CERT_ORG_NAME: &str = "spec-ai";

/// Common name prefix for the certificate
const CERT_CN_PREFIX: &str = "spec-ai-server";

/// TLS configuration and certificate info
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// The generated or loaded certificate (DER format)
    pub certificate: Vec<u8>,
    /// The private key (DER format)
    pub private_key: Vec<u8>,
    /// SHA-256 fingerprint of the certificate (hex encoded)
    pub fingerprint: String,
    /// Certificate in PEM format (for export/display)
    pub certificate_pem: String,
    /// When the certificate expires
    pub not_after: String,
}

/// Certificate metadata returned to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// SHA-256 fingerprint of the certificate (hex encoded)
    pub fingerprint: String,
    /// Certificate in PEM format
    pub certificate_pem: String,
    /// When the certificate was issued
    pub not_before: String,
    /// When the certificate expires
    pub not_after: String,
    /// Subject common name
    pub subject: String,
    /// Subject alternative names
    pub san: Vec<String>,
}

impl TlsConfig {
    /// Generate a new self-signed certificate
    ///
    /// # Arguments
    /// * `hostname` - Primary hostname for the certificate
    /// * `additional_sans` - Additional Subject Alternative Names (IPs, hostnames)
    /// * `validity_days` - Certificate validity period in days
    pub fn generate(
        hostname: &str,
        additional_sans: &[String],
        validity_days: Option<u32>,
    ) -> Result<Self> {
        let validity = validity_days.unwrap_or(DEFAULT_CERT_VALIDITY_DAYS);

        // Generate key pair
        let key_pair = KeyPair::generate().context("Failed to generate key pair")?;

        // Build certificate parameters
        let mut params = CertificateParams::default();

        // Set distinguished name
        params
            .distinguished_name
            .push(DnType::OrganizationName, CERT_ORG_NAME);
        params.distinguished_name.push(
            DnType::CommonName,
            format!("{}-{}", CERT_CN_PREFIX, hostname),
        );

        // Set validity period
        let now = time::OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::from_secs(validity as u64 * 24 * 60 * 60);

        // Set key usages for TLS server
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

        // Not a CA certificate
        params.is_ca = IsCa::NoCa;

        // Add Subject Alternative Names
        let mut sans = vec![SanType::DnsName(
            hostname.try_into().context("Invalid hostname")?,
        )];

        // Always add localhost variants
        if hostname != "localhost" {
            if let Ok(localhost) = "localhost".try_into() {
                sans.push(SanType::DnsName(localhost));
            }
        }

        // Add 127.0.0.1 as IP SAN
        sans.push(SanType::IpAddress(std::net::IpAddr::V4(
            std::net::Ipv4Addr::new(127, 0, 0, 1),
        )));

        // Add additional SANs
        for san in additional_sans {
            if let Ok(ip) = san.parse::<std::net::IpAddr>() {
                sans.push(SanType::IpAddress(ip));
            } else if let Ok(dns) = san.as_str().try_into() {
                sans.push(SanType::DnsName(dns));
            }
        }

        params.subject_alt_names = sans;

        // Save not_after before consuming params
        let not_after_time = params.not_after;

        // Generate the certificate
        let cert = params
            .self_signed(&key_pair)
            .context("Failed to generate self-signed certificate")?;

        // Get DER-encoded certificate and key
        let cert_der = cert.der().to_vec();
        let key_der = key_pair.serialize_der();

        // Calculate fingerprint
        let fingerprint = Self::calculate_fingerprint(&cert_der);

        // Get PEM format
        let cert_pem = cert.pem();

        // Format expiry date
        let not_after = not_after_time
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".to_string());

        tracing::info!(
            "Generated self-signed TLS certificate for {} (fingerprint: {})",
            hostname,
            fingerprint
        );

        Ok(Self {
            certificate: cert_der,
            private_key: key_der,
            fingerprint,
            certificate_pem: cert_pem,
            not_after,
        })
    }

    /// Load certificate and key from PEM files
    pub fn load_from_files(cert_path: &Path, key_path: &Path) -> Result<Self> {
        let cert_pem = std::fs::read_to_string(cert_path)
            .with_context(|| format!("Failed to read certificate file: {}", cert_path.display()))?;

        let key_pem = std::fs::read_to_string(key_path)
            .with_context(|| format!("Failed to read key file: {}", key_path.display()))?;

        Self::load_from_pem(&cert_pem, &key_pem)
    }

    /// Load certificate and key from PEM strings
    pub fn load_from_pem(cert_pem: &str, key_pem: &str) -> Result<Self> {
        // Parse certificate
        let mut cert_reader = std::io::BufReader::new(cert_pem.as_bytes());
        let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse certificate PEM")?;

        let cert_der = certs
            .into_iter()
            .next()
            .context("No certificate found in PEM")?;

        // Parse private key
        let mut key_reader = std::io::BufReader::new(key_pem.as_bytes());
        let key_der = rustls_pemfile::private_key(&mut key_reader)
            .context("Failed to parse private key PEM")?
            .context("No private key found in PEM")?;

        let fingerprint = Self::calculate_fingerprint(cert_der.as_ref());

        Ok(Self {
            certificate: cert_der.to_vec(),
            private_key: match key_der {
                PrivateKeyDer::Pkcs8(k) => k.secret_pkcs8_der().to_vec(),
                PrivateKeyDer::Pkcs1(k) => k.secret_pkcs1_der().to_vec(),
                PrivateKeyDer::Sec1(k) => k.secret_sec1_der().to_vec(),
                _ => anyhow::bail!("Unsupported private key format"),
            },
            fingerprint,
            certificate_pem: cert_pem.to_string(),
            not_after: "unknown".to_string(), // Would need to parse cert to get this
        })
    }

    /// Save certificate and key to PEM files
    pub fn save_to_files(&self, cert_path: &Path, key_path: &Path) -> Result<()> {
        // Ensure parent directories exist
        if let Some(parent) = cert_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Some(parent) = key_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save certificate PEM
        std::fs::write(cert_path, &self.certificate_pem)
            .with_context(|| format!("Failed to write certificate to {}", cert_path.display()))?;

        // Convert private key to PEM and save
        let key_pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
            STANDARD.encode(&self.private_key)
        );
        std::fs::write(key_path, &key_pem)
            .with_context(|| format!("Failed to write private key to {}", key_path.display()))?;

        // Set restrictive permissions on key file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        tracing::info!(
            "Saved TLS certificate to {} and key to {}",
            cert_path.display(),
            key_path.display()
        );

        Ok(())
    }

    /// Calculate SHA-256 fingerprint of a certificate (DER format)
    pub fn calculate_fingerprint(cert_der: &[u8]) -> String {
        let hash = digest(&SHA256, cert_der);
        hash.as_ref()
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":")
    }

    /// Get certificate info for clients
    pub fn get_certificate_info(&self, hostname: &str) -> CertificateInfo {
        CertificateInfo {
            fingerprint: self.fingerprint.clone(),
            certificate_pem: self.certificate_pem.clone(),
            not_before: "see certificate".to_string(),
            not_after: self.not_after.clone(),
            subject: format!("CN={}-{}, O={}", CERT_CN_PREFIX, hostname, CERT_ORG_NAME),
            san: vec![
                hostname.to_string(),
                "localhost".to_string(),
                "127.0.0.1".to_string(),
            ],
        }
    }

    /// Build rustls ServerConfig from this TLS config
    pub fn build_server_config(&self) -> Result<Arc<rustls::ServerConfig>> {
        let cert = CertificateDer::from(self.certificate.clone());
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.private_key.clone()));

        // Use aws-lc-rs as the crypto provider (installed by default via axum-server)
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .context("Failed to set protocol versions")?
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .context("Failed to build TLS server config")?;

        Ok(Arc::new(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_certificate() {
        let config = TlsConfig::generate("test.local", &[], Some(30)).unwrap();

        assert!(!config.certificate.is_empty());
        assert!(!config.private_key.is_empty());
        assert!(!config.fingerprint.is_empty());
        assert!(config.certificate_pem.contains("BEGIN CERTIFICATE"));

        // Fingerprint should be colon-separated hex
        assert!(config.fingerprint.contains(':'));
        let parts: Vec<&str> = config.fingerprint.split(':').collect();
        assert_eq!(parts.len(), 32); // SHA-256 = 32 bytes
    }

    #[test]
    fn test_fingerprint_calculation() {
        let data = b"test certificate data";
        let fingerprint = TlsConfig::calculate_fingerprint(data);

        // Should be 32 hex pairs separated by colons
        let parts: Vec<&str> = fingerprint.split(':').collect();
        assert_eq!(parts.len(), 32);

        // Each part should be 2 hex chars
        for part in parts {
            assert_eq!(part.len(), 2);
            assert!(part.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn test_build_server_config() {
        let tls = TlsConfig::generate("localhost", &[], None).unwrap();
        let server_config = tls.build_server_config();
        assert!(server_config.is_ok());
    }

    #[test]
    fn test_additional_sans() {
        let additional = vec!["192.168.1.100".to_string(), "myserver.local".to_string()];
        let config = TlsConfig::generate("primary.local", &additional, None).unwrap();

        assert!(!config.certificate.is_empty());
        // The SANs are embedded in the certificate - we'd need to parse it to verify
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        // Generate and save
        let original = TlsConfig::generate("test.local", &[], None).unwrap();
        original.save_to_files(&cert_path, &key_path).unwrap();

        // Load and verify
        let loaded = TlsConfig::load_from_files(&cert_path, &key_path).unwrap();

        assert_eq!(original.certificate, loaded.certificate);
        assert_eq!(original.fingerprint, loaded.fingerprint);
    }
}
