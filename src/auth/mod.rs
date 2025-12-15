mod error;
mod loader;
pub mod middleware; // Make public for testing
mod types;

// Re-export public types
pub use types::{
    AuthenticatedUser, CredentialsConfig, CredentialsStore, UserConfig, UserCredentials,
    validate_api_key,
};

// Re-export middleware types
pub use middleware::AuthLayer;
pub use error::AuthError; // Re-export for testing

// Re-export loader
pub use loader::{get_credentials_path, load_credentials};
