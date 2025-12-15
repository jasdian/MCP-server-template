use anyhow::{Context, Result};
use mcp_server::auth::load_credentials;
use mcp_server::create_app;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

/// Setup and configure the MCP server application
///
/// Handles loading credentials and creating the configured Axum router.
pub fn setup_server() -> Result<axum::Router> {
    let credentials = load_credentials().context("Failed to load credentials")?;
    let app = create_app(credentials);
    Ok(app)
}

#[tokio::main]
async fn main() {
    let app = setup_server().expect("Failed to setup server");

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    println!("MCP Server listening on http://{}", addr);
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    fn get_credentials_example_path() -> PathBuf {
        // Use CARGO_MANIFEST_DIR to get the project root, which works in both
        // regular test runs and tarpaulin runs
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR should be set during tests");
        PathBuf::from(manifest_dir).join("config/credentials.toml.example")
    }

    #[test]
    fn test_setup_server_with_valid_credentials() {
        // Use example credentials file (checked into git)
        let example_path = get_credentials_example_path();
        unsafe {
            env::set_var("MCP_CREDENTIALS_PATH", example_path);
        }

        let result = setup_server();
        assert!(
            result.is_ok(),
            "setup_server should succeed with valid credentials"
        );

        unsafe {
            env::remove_var("MCP_CREDENTIALS_PATH");
        }
    }

    #[test]
    fn test_setup_server_returns_router() {
        let example_path = get_credentials_example_path();
        unsafe {
            env::set_var("MCP_CREDENTIALS_PATH", example_path);
        }

        let result = setup_server();
        assert!(result.is_ok());

        // Verify we get a Router back
        let _router: axum::Router = result.unwrap();

        unsafe {
            env::remove_var("MCP_CREDENTIALS_PATH");
        }
    }
}
