use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

/// TOML configuration structure for credentials file
/// Uses flatten to map username keys directly to UserConfig
#[derive(Debug, Deserialize)]
pub struct CredentialsConfig {
    #[serde(flatten)]
    pub users: HashMap<String, UserConfig>,
}

/// Individual user configuration from TOML
#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub api_key: String,
    #[serde(default)]
    pub external_keys: HashMap<String, String>,
}

/// Runtime user credentials with username, API key, and external service keys
#[derive(Debug, Clone)]
pub struct UserCredentials {
    pub username: String,
    pub api_key: String,
    pub external_keys: HashMap<String, String>,
}

impl UserCredentials {
    /// Create new UserCredentials
    pub fn new(username: String, api_key: String, external_keys: HashMap<String, String>) -> Self {
        Self {
            username,
            api_key,
            external_keys,
        }
    }

    /// Get an external service key (e.g., "postgres_url", "stripe_key")
    pub fn get_external_key(&self, key: &str) -> Option<&String> {
        self.external_keys.get(key)
    }
}

/// Credentials store indexed by API key for O(1) lookups
/// HashMap<api_key, UserCredentials>
pub type CredentialsStoreInner = HashMap<String, UserCredentials>;

/// Thread-safe credentials store
pub type CredentialsStore = Arc<CredentialsStoreInner>;

/// Wrapper for authenticated user context
/// Injected into request extensions by auth middleware
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub UserCredentials);

impl AuthenticatedUser {
    /// Get reference to underlying UserCredentials
    pub fn credentials(&self) -> &UserCredentials {
        &self.0
    }

    /// Get external service key
    pub fn get_external_key(&self, key: &str) -> Option<&String> {
        self.0.get_external_key(key)
    }
}

/// Validate API key against credentials store
/// Returns UserCredentials if valid, None otherwise
pub fn validate_api_key(api_key: &str, store: &CredentialsStore) -> Option<UserCredentials> {
    store.get(api_key).cloned()
}
