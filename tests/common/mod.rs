use anyhow::{Error, Result};
use mcp_server::auth::AuthenticatedUser;
use mcp_server::auth::{CredentialsStore, UserCredentials};
use mcp_server::tools::McpTool;
use mcp_server::tools::PinBoxedFuture;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

// Test constants
pub const TEST_API_KEY: &str = "test-api-key-12345";
pub const TEST_API_KEY_2: &str = "test-api-key-67890";
pub const TEST_USERNAME: &str = "testuser";
pub const TEST_USERNAME_2: &str = "testuser2";

/// Create a test credentials store with one user
pub fn create_test_credentials_store() -> CredentialsStore {
    let mut store = HashMap::new();
    store.insert(
        TEST_API_KEY.to_string(),
        UserCredentials::new(
            TEST_USERNAME.to_string(),
            TEST_API_KEY.to_string(),
            HashMap::new(),
        ),
    );
    Arc::new(store)
}

/// Create a test credentials store with multiple users
pub fn create_multi_user_credentials_store() -> CredentialsStore {
    let mut store = HashMap::new();

    store.insert(
        TEST_API_KEY.to_string(),
        UserCredentials::new(
            TEST_USERNAME.to_string(),
            TEST_API_KEY.to_string(),
            HashMap::new(),
        ),
    );

    let mut external_keys = HashMap::new();
    external_keys.insert(
        "postgres_url".to_string(),
        "postgresql://localhost/test".to_string(),
    );
    external_keys.insert("api_key".to_string(), "external-api-key".to_string());

    store.insert(
        TEST_API_KEY_2.to_string(),
        UserCredentials::new(
            TEST_USERNAME_2.to_string(),
            TEST_API_KEY_2.to_string(),
            external_keys,
        ),
    );

    Arc::new(store)
}

/// Create a test user with no external keys
pub fn create_test_user() -> UserCredentials {
    UserCredentials::new(
        TEST_USERNAME.to_string(),
        TEST_API_KEY.to_string(),
        HashMap::new(),
    )
}

/// Create a test user with external keys
pub fn create_test_user_with_external_keys() -> UserCredentials {
    let mut external_keys = HashMap::new();
    external_keys.insert(
        "postgres_url".to_string(),
        "postgresql://localhost/testdb".to_string(),
    );
    external_keys.insert("stripe_key".to_string(), "sk_test_123".to_string());

    UserCredentials::new(
        TEST_USERNAME.to_string(),
        TEST_API_KEY.to_string(),
        external_keys,
    )
}

/// Create a temporary credentials TOML file with valid content
#[allow(dead_code)]
pub fn create_temp_credentials_file() -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    writeln!(
        file,
        r#"[alice]
api_key = "alice-key-123"

[bob]
api_key = "bob-key-456"

[bob.external_keys]
postgres_url = "postgresql://localhost/bobdb"
"#
    )?;
    file.flush()?;
    Ok(file)
}

/// Create a temporary credentials file with a single user
#[allow(dead_code)]
pub fn create_temp_single_user_file() -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    writeln!(
        file,
        r#"[alice]
api_key = "alice-key-123"
"#
    )?;
    file.flush()?;
    Ok(file)
}

/// Create a temporary credentials file with duplicate API keys
#[allow(dead_code)]
pub fn create_temp_duplicate_keys_file() -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    writeln!(
        file,
        r#"[alice]
api_key = "duplicate-key"

[bob]
api_key = "duplicate-key"
"#
    )?;
    file.flush()?;
    Ok(file)
}

/// Create a temporary credentials file with invalid TOML
#[allow(dead_code)]
pub fn create_temp_invalid_toml_file() -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "invalid [ toml syntax")?;
    file.flush()?;
    Ok(file)
}

/// Create a temporary empty credentials file
pub fn create_temp_empty_file() -> Result<NamedTempFile> {
    let file = NamedTempFile::new()?;
    Ok(file)
}

/// Mock tool for testing
pub struct MockTool {
    pub name: &'static str,
    pub description: &'static str,
}

impl McpTool for MockTool {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "test_param": {
                    "type": "string",
                    "description": "Test parameter"
                }
            },
            "required": [],
            "additionalProperties": false
        })
    }

    fn execute(
        &self,
        _args: Option<Value>,
        _user: AuthenticatedUser,
    ) -> PinBoxedFuture<Result<Value, Error>> {
        Box::pin(async move { Ok(json!({"result": "mock_result"})) })
    }
}

impl MockTool {
    pub fn new(name: &'static str, description: &'static str) -> Self {
        Self { name, description }
    }
}
