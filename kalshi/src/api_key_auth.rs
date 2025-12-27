use base64::{engine::general_purpose::STANDARD, Engine};
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::{Signature, SigningKey};
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use rsa::RsaPrivateKey;
use sha2::Sha256;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::kalshi_error::KalshiError;

/// Holds API key credentials for Kalshi authentication.
#[derive(Debug, Clone)]
pub struct KalshiCredentials {
    pub key_id: String,
    pub private_key: RsaPrivateKey,
}

impl KalshiCredentials {
    /// Creates credentials from an API key ID and PEM content or file path.
    /// Automatically detects whether the input is a file path or raw PEM content.
    pub fn new(key_id: &str, pem_path_or_content: &str) -> Result<Self, KalshiError> {
        let private_key = load_private_key(pem_path_or_content)?;
        Ok(Self {
            key_id: key_id.to_string(),
            private_key,
        })
    }

    /// Creates credentials from environment variables.
    /// The PEM env var can contain either a file path or raw PEM content.
    pub fn from_env(key_id_var: &str, pem_var: &str) -> Result<Self, KalshiError> {
        let key_id = std::env::var(key_id_var).map_err(|_| {
            KalshiError::UserInputError(format!(
                "Environment variable '{}' not set",
                key_id_var
            ))
        })?;

        let pem_value = std::env::var(pem_var).map_err(|_| {
            KalshiError::UserInputError(format!("Environment variable '{}' not set", pem_var))
        })?;

        Self::new(&key_id, &pem_value)
    }

    /// Generates the authentication headers for a request.
    /// Returns (key_id, timestamp_ms, signature_base64).
    pub fn generate_auth_headers(
        &self,
        method: &str,
        path: &str,
    ) -> Result<(String, String, String), KalshiError> {
        let timestamp_ms = current_timestamp_ms();
        let signature = generate_signature(&self.private_key, timestamp_ms, method, path)?;

        Ok((
            self.key_id.clone(),
            timestamp_ms.to_string(),
            signature,
        ))
    }
}

/// Loads a private key from either a file path or raw PEM content.
fn load_private_key(pem_path_or_content: &str) -> Result<RsaPrivateKey, KalshiError> {
    let pem_content = if looks_like_pem(pem_path_or_content) {
        pem_path_or_content.to_string()
    } else if Path::new(pem_path_or_content).exists() {
        fs::read_to_string(pem_path_or_content).map_err(|e| {
            KalshiError::UserInputError(format!(
                "Failed to read PEM file '{}': {}",
                pem_path_or_content, e
            ))
        })?
    } else {
        return Err(KalshiError::UserInputError(format!(
            "PEM input is neither valid PEM content nor an existing file path: '{}'",
            pem_path_or_content
        )));
    };

    RsaPrivateKey::from_pkcs8_pem(&pem_content).map_err(|e| {
        KalshiError::UserInputError(format!("Failed to parse PEM private key: {}", e))
    })
}

/// Checks if a string looks like PEM content (starts with -----BEGIN).
fn looks_like_pem(s: &str) -> bool {
    s.trim().starts_with("-----BEGIN")
}

/// Returns current timestamp in milliseconds since Unix epoch.
fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

/// Generates RSA-PSS signature for Kalshi API authentication.
/// Signs: timestamp_ms + method + path (path without query params).
fn generate_signature(
    private_key: &RsaPrivateKey,
    timestamp_ms: i64,
    method: &str,
    path: &str,
) -> Result<String, KalshiError> {
    // Build message to sign: timestamp + method + path
    let message = format!("{}{}{}", timestamp_ms, method, path);

    // Create PSS signing key with SHA256
    let signing_key = SigningKey::<Sha256>::new(private_key.clone());

    // Sign with randomized PSS padding
    let mut rng = rand::thread_rng();
    let signature: Signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());

    // Base64 encode the signature
    Ok(STANDARD.encode(signature.to_bytes()))
}

/// Extracts the path from a URL, stripping query parameters.
pub fn extract_path_for_signing(url: &reqwest::Url) -> String {
    url.path().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_pem() {
        assert!(looks_like_pem("-----BEGIN PRIVATE KEY-----\ntest"));
        assert!(looks_like_pem("  -----BEGIN RSA PRIVATE KEY-----"));
        assert!(!looks_like_pem("/path/to/key.pem"));
        assert!(!looks_like_pem("some random string"));
    }

    #[test]
    fn test_current_timestamp_ms() {
        let ts = current_timestamp_ms();
        assert!(ts > 1700000000000); // After Nov 2023
    }
}
