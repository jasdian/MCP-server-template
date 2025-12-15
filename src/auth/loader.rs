use super::types::{CredentialsConfig, CredentialsStore, UserCredentials};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Arc;

/// Get the path to the credentials file
/// Checks MCP_CREDENTIALS_PATH env var, defaults to config/credentials.toml
pub fn get_credentials_path() -> String {
    env::var("MCP_CREDENTIALS_PATH").unwrap_or_else(|_| "config/credentials.toml".to_string())
}

/// Load credentials from TOML file
/// Returns Arc-wrapped HashMap indexed by API key
pub fn load_credentials() -> Result<CredentialsStore> {
    let path = get_credentials_path();

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read credentials file at: {}", path))?;
    let config: CredentialsConfig = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse credentials file at: {}", path))?;

    if config.users.is_empty() {
        anyhow::bail!("No users found in credentials file at: {}", path);
    }

    let mut store = HashMap::new();
    for (username, user_config) in config.users {
        let credentials = UserCredentials::new(
            username.clone(),
            user_config.api_key.clone(),
            user_config.external_keys,
        );

        if store.contains_key(&user_config.api_key) {
            anyhow::bail!(
                "Duplicate API key found for user '{}': {}",
                username,
                user_config.api_key
            );
        }

        store.insert(user_config.api_key, credentials);
    }

    Ok(Arc::new(store))
}
