mod common;

use common::*;
use mcp_server::auth::{get_credentials_path, load_credentials};
use std::env;
use std::sync::Mutex;

// Mutex to ensure env var tests run sequentially (prevents race conditions)
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_get_credentials_path_default() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Remove env var if it exists
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }

    let path = get_credentials_path();
    assert_eq!(path, "config/credentials.toml");
}

#[test]
fn test_get_credentials_path_from_env() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let custom_path = "/custom/path/credentials.toml";
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", custom_path); }

    let path = get_credentials_path();
    assert_eq!(path, custom_path);

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_valid_file() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_credentials_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_ok());

    let store = result.unwrap();
    assert_eq!(store.len(), 2);
    assert!(store.contains_key("alice-key-123"));
    assert!(store.contains_key("bob-key-456"));

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_file_not_found() {
    let _lock = ENV_MUTEX.lock().unwrap();

    unsafe { env::set_var("MCP_CREDENTIALS_PATH", "/nonexistent/path/file.toml"); }

    let result = load_credentials();
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Failed to read credentials file"));

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_invalid_toml() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_invalid_toml_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Failed to parse credentials file"));

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_empty_user_list() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_empty_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("No users found"));

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_duplicate_api_keys() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_duplicate_keys_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Duplicate API key"));

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_single_user() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_single_user_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_ok());

    let store = result.unwrap();
    assert_eq!(store.len(), 1);
    assert!(store.contains_key("alice-key-123"));

    let alice = store.get("alice-key-123").unwrap();
    assert_eq!(alice.username, "alice");
    assert_eq!(alice.api_key, "alice-key-123");
    assert_eq!(alice.external_keys.len(), 0);

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_with_external_keys() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_credentials_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_ok());

    let store = result.unwrap();

    // Bob should have external keys
    let bob = store.get("bob-key-456").unwrap();
    assert_eq!(bob.username, "bob");
    assert_eq!(bob.external_keys.len(), 1);
    assert_eq!(
        bob.external_keys.get("postgres_url"),
        Some(&"postgresql://localhost/bobdb".to_string())
    );

    // Alice should have no external keys
    let alice = store.get("alice-key-123").unwrap();
    assert_eq!(alice.external_keys.len(), 0);

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_indexed_by_api_key() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_credentials_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_ok());

    let store = result.unwrap();

    // Verify store is indexed by API key, not username
    assert!(!store.contains_key("alice"));  // username not in keys
    assert!(!store.contains_key("bob"));    // username not in keys
    assert!(store.contains_key("alice-key-123"));  // api_key is key
    assert!(store.contains_key("bob-key-456"));    // api_key is key

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_arc_wrapped() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_single_user_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_ok());

    let store = result.unwrap();

    // Verify it's Arc-wrapped by being able to clone it cheaply
    let store_clone = store.clone();
    assert_eq!(store.len(), store_clone.len());

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}

#[test]
fn test_load_credentials_preserves_usernames() {
    let _lock = ENV_MUTEX.lock().unwrap();

    let file = create_temp_credentials_file().expect("Failed to create temp file");
    let path_str = file.path().to_str().unwrap();
    unsafe { env::set_var("MCP_CREDENTIALS_PATH", path_str); }

    let result = load_credentials();
    assert!(result.is_ok());

    let store = result.unwrap();

    // Even though indexed by API key, UserCredentials should preserve username
    let alice = store.get("alice-key-123").unwrap();
    assert_eq!(alice.username, "alice");

    let bob = store.get("bob-key-456").unwrap();
    assert_eq!(bob.username, "bob");

    // Cleanup
    unsafe { env::remove_var("MCP_CREDENTIALS_PATH"); }
}
