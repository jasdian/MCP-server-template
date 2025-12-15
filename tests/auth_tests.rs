mod common;

use common::*;
use mcp_server::auth::{validate_api_key, AuthenticatedUser, UserCredentials};
use mcp_server::auth::{CredentialsConfig, UserConfig};
use std::collections::HashMap;

#[test]
fn test_user_credentials_new() {
    let username = "alice";
    let api_key = "alice-key";
    let mut external_keys = HashMap::new();
    external_keys.insert("db_url".to_string(), "postgresql://localhost/db".to_string());

    let creds = UserCredentials::new(
        username.to_string(),
        api_key.to_string(),
        external_keys.clone(),
    );

    assert_eq!(creds.username, username);
    assert_eq!(creds.api_key, api_key);
    assert_eq!(creds.external_keys.len(), 1);
    assert_eq!(
        creds.external_keys.get("db_url"),
        Some(&"postgresql://localhost/db".to_string())
    );
}

#[test]
fn test_user_credentials_get_external_key_exists() {
    let mut external_keys = HashMap::new();
    external_keys.insert("postgres_url".to_string(), "postgresql://localhost/testdb".to_string());
    external_keys.insert("stripe_key".to_string(), "sk_test_123".to_string());

    let creds = UserCredentials::new(
        "testuser".to_string(),
        "test-key".to_string(),
        external_keys,
    );

    assert_eq!(
        creds.get_external_key("postgres_url"),
        Some(&"postgresql://localhost/testdb".to_string())
    );
    assert_eq!(
        creds.get_external_key("stripe_key"),
        Some(&"sk_test_123".to_string())
    );
}

#[test]
fn test_user_credentials_get_external_key_not_exists() {
    let creds = UserCredentials::new(
        "testuser".to_string(),
        "test-key".to_string(),
        HashMap::new(),
    );

    assert_eq!(creds.get_external_key("nonexistent_key"), None);
}

#[test]
fn test_user_credentials_no_external_keys() {
    let creds = create_test_user();
    assert_eq!(creds.external_keys.len(), 0);
    assert_eq!(creds.get_external_key("any_key"), None);
}

#[test]
fn test_authenticated_user_credentials() {
    let user_creds = create_test_user();
    let auth_user = AuthenticatedUser(user_creds.clone());

    assert_eq!(auth_user.credentials().username, user_creds.username);
    assert_eq!(auth_user.credentials().api_key, user_creds.api_key);
}

#[test]
fn test_authenticated_user_get_external_key() {
    let user_creds = create_test_user_with_external_keys();
    let auth_user = AuthenticatedUser(user_creds);

    assert_eq!(
        auth_user.get_external_key("postgres_url"),
        Some(&"postgresql://localhost/testdb".to_string())
    );
    assert_eq!(
        auth_user.get_external_key("stripe_key"),
        Some(&"sk_test_123".to_string())
    );
    assert_eq!(auth_user.get_external_key("nonexistent"), None);
}

#[test]
fn test_validate_api_key_valid() {
    let store = create_test_credentials_store();
    let result = validate_api_key(TEST_API_KEY, &store);

    assert!(result.is_some());
    let creds = result.unwrap();
    assert_eq!(creds.username, TEST_USERNAME);
    assert_eq!(creds.api_key, TEST_API_KEY);
}

#[test]
fn test_validate_api_key_invalid() {
    let store = create_test_credentials_store();
    let result = validate_api_key("invalid-key", &store);

    assert!(result.is_none());
}

#[test]
fn test_validate_api_key_empty_store() {
    let store = std::sync::Arc::new(HashMap::new());
    let result = validate_api_key(TEST_API_KEY, &store);

    assert!(result.is_none());
}

#[test]
fn test_validate_api_key_multiple_users() {
    let store = create_multi_user_credentials_store();

    // Validate first user
    let result1 = validate_api_key(TEST_API_KEY, &store);
    assert!(result1.is_some());
    let creds1 = result1.unwrap();
    assert_eq!(creds1.username, TEST_USERNAME);

    // Validate second user
    let result2 = validate_api_key(TEST_API_KEY_2, &store);
    assert!(result2.is_some());
    let creds2 = result2.unwrap();
    assert_eq!(creds2.username, TEST_USERNAME_2);
    assert_eq!(creds2.external_keys.len(), 2);
}

#[test]
fn test_credentials_config_deserialization_single_user() {
    let toml_str = r#"
[alice]
api_key = "alice-key-123"
"#;

    let config: CredentialsConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
    assert_eq!(config.users.len(), 1);
    assert!(config.users.contains_key("alice"));
    assert_eq!(config.users["alice"].api_key, "alice-key-123");
    assert_eq!(config.users["alice"].external_keys.len(), 0);
}

#[test]
fn test_credentials_config_deserialization_multiple_users() {
    let toml_str = r#"
[alice]
api_key = "alice-key-123"

[bob]
api_key = "bob-key-456"
"#;

    let config: CredentialsConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
    assert_eq!(config.users.len(), 2);
    assert!(config.users.contains_key("alice"));
    assert!(config.users.contains_key("bob"));
}

#[test]
fn test_credentials_config_with_external_keys() {
    let toml_str = r#"
[alice]
api_key = "alice-key-123"

[alice.external_keys]
postgres_url = "postgresql://localhost/alicedb"
stripe_key = "sk_test_alice"
"#;

    let config: CredentialsConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
    assert_eq!(config.users.len(), 1);

    let alice = &config.users["alice"];
    assert_eq!(alice.api_key, "alice-key-123");
    assert_eq!(alice.external_keys.len(), 2);
    assert_eq!(
        alice.external_keys.get("postgres_url"),
        Some(&"postgresql://localhost/alicedb".to_string())
    );
    assert_eq!(
        alice.external_keys.get("stripe_key"),
        Some(&"sk_test_alice".to_string())
    );
}

#[test]
fn test_credentials_config_mixed_external_keys() {
    let toml_str = r#"
[alice]
api_key = "alice-key-123"

[alice.external_keys]
db_url = "postgresql://localhost/alicedb"

[bob]
api_key = "bob-key-456"
"#;

    let config: CredentialsConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
    assert_eq!(config.users.len(), 2);

    assert_eq!(config.users["alice"].external_keys.len(), 1);
    assert_eq!(config.users["bob"].external_keys.len(), 0);
}

#[test]
fn test_credentials_config_invalid_toml() {
    let toml_str = "invalid [ toml syntax";
    let result: Result<CredentialsConfig, _> = toml::from_str(toml_str);
    assert!(result.is_err());
}

#[test]
fn test_user_config_default_external_keys() {
    // When external_keys is not specified, it should default to empty HashMap
    let toml_str = r#"
api_key = "test-key"
"#;

    let config: UserConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
    assert_eq!(config.api_key, "test-key");
    assert_eq!(config.external_keys.len(), 0);
}

#[test]
fn test_credentials_clone() {
    let creds = create_test_user_with_external_keys();
    let cloned = creds.clone();

    assert_eq!(creds.username, cloned.username);
    assert_eq!(creds.api_key, cloned.api_key);
    assert_eq!(creds.external_keys.len(), cloned.external_keys.len());
}

#[test]
fn test_authenticated_user_clone() {
    let user_creds = create_test_user();
    let auth_user = AuthenticatedUser(user_creds);
    let cloned = auth_user.clone();

    assert_eq!(
        auth_user.credentials().username,
        cloned.credentials().username
    );
}
