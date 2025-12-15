# MCP Server

A minimal MCP (Model Context Protocol) server template built with Rust, Axum, and Tower. This template provides authentication, tool registration, and a clean architecture for building custom MCP servers.

> [!WARNING]
> You connect this server to your MCP client at your own responsibility. Language models can make mistakes, misinterpret instructions, or perform unintended actions. Always verify commands before execution, especially for database operations (INSERT, UPDATE, DELETE), Python script execution, or operations using external API keys from your credentials.
>
> The current authentication system stores API keys in plain text and is suitable for development and internal use. For production deployments, implement additional security measures: API key hashing (bcrypt/argon2), TLS/HTTPS termination, rate limiting, audit logging, secure credential storage, and response caching to prevent excessive API calls.

## Features

- **Authentication**: Bearer token authentication with per-user credentials
- **MCP Protocol**: JSON-RPC 2.0 compliant implementation
- **Tool System**: Trait-based tool registration with JSON Schema validation
- **Type Safety**: Leverages Rust's type system for compile-time guarantees
- **Performance**: Arc-wrapped state, O(1) API key lookups
- **Security**: Middleware-based authentication, per-user external credentials
- **Extensibility**: Simple trait implementation to add new tools

## Architecture

```text
mcp-server/
├── src/
│   ├── main.rs              # Server entry point
│   ├── lib.rs               # Core MCP protocol implementation
│   ├── auth/                # Authentication module
│   │   ├── mod.rs          # Module exports
│   │   ├── types.rs        # Credential data structures
│   │   ├── middleware.rs   # Tower authentication layer
│   │   ├── error.rs        # Auth-specific errors
│   │   └── loader.rs       # TOML credentials loading
│   └── tools/               # Tool implementations
│       ├── mod.rs          # McpTool trait and registration
│       └── get_time.rs     # Example tool
├── config/
│   └── credentials.toml    # User credentials (not in git)
└── Cargo.toml              # Dependencies
```

### Core Components

- **MCP Protocol Handler** (`lib.rs`): Handles `discover` and `invoke` requests
- **Authentication Layer** (`auth/`): Tower middleware for Bearer token validation
- **Tool Registry** (`tools/`): Trait-based system for registering and executing tools
- **Credentials Store** (`auth/types.rs`): HashMap indexed by API key for O(1) lookups

## Quick Start

### 1. Clone and Setup

```bash
git clone <repository-url> mcp-server
cd mcp-server
```

### 2. Configure Credentials

```bash
cp config/credentials.toml.example config/credentials.toml
```

Edit `config/credentials.toml` and add your API keys:

```toml
[alice]
api_key = "your-secure-api-key-here"

[alice.external_keys]
# Add external service credentials if needed
```

**Important**: Add `config/credentials.toml` to `.gitignore` to avoid committing secrets!

### 3. Build and Run

```bash
cargo build --release
cargo run
```

The server will start on `http://0.0.0.0:3000`.

### 4. Test the Server

**Health Check:**

```bash
curl http://localhost:3000/health
```

**Discover Tools:**

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"method": "discover"}'
```

**Invoke a Tool:**

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"method": "invoke", "params": {"tool_name": "get_current_time"}}'
```

## Configuration

### Environment Variables

- `MCP_CREDENTIALS_PATH`: Path to credentials file (default: `config/credentials.toml`)

### Credentials File Format

```toml
[username]
api_key = "bearer-token-for-authentication"

[username.external_keys]
# Optional: External service credentials for this user
postgres_url = "postgresql://localhost/dbname"
stripe_key = "sk_test_..."
```

## API Documentation

### Endpoints

#### `GET /health`

Health check endpoint (no authentication required).

**Response:**

```text
OK
```

#### `POST /mcp`

Main MCP endpoint (requires Bearer authentication).

**Request Format:**

```json
{
  "method": "discover" | "invoke",
  "params": { ... }
}
```

### MCP Methods

#### `discover`

Returns a list of all available tools.

**Request:**

```json
{
  "method": "discover"
}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "name": "get_current_time",
        "description": "Returns the current server time as an ISO 8601 string.",
        "parameters": {
          "type": "object",
          "properties": {},
          "additionalProperties": false,
          "required": []
        }
      }
    ]
  }
}
```

#### `invoke`

Executes a specific tool.

**Request:**

```json
{
  "method": "invoke",
  "params": {
    "tool_name": "get_current_time",
    "arguments": {}
  }
}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "current_time": "2025-12-15T10:30:00.123456789Z"
  }
}
```

### Error Codes

MCP Server uses JSON-RPC 2.0 error codes:

| Code    | Name                  | Description                           |
|---------|-----------------------|---------------------------------------|
| -32001  | ERROR_AUTH            | Authentication failure                |
| -32002  | ERROR_INVALID_PARAMS  | Invalid or missing parameters         |
| -32003  | ERROR_TOOL_EXECUTION  | Tool execution error                  |
| -32600  | ERROR_INVALID_REQUEST | Malformed request                     |
| -32601  | ERROR_METHOD_NOT_FOUND| Tool not found                        |

**Error Response Example:**

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32601,
    "message": "Tool 'unknown_tool' not found",
    "data": {
      "available_tools": ["get_current_time"]
    }
  }
}
```

## Adding New Tools

### 1. Create a New Tool Module

Create a new file in `src/tools/`, e.g., `src/tools/my_tool.rs`:

```rust
use super::{validate_tool_args, McpTool, PinBoxedFuture};
use crate::auth::AuthenticatedUser;
use anyhow::{Error, Result};
use serde_json::{json, Value};

pub struct MyTool;

impl McpTool for MyTool {
    fn name(&self) -> &'static str {
        "my_tool"
    }

    fn description(&self) -> &'static str {
        "Description of what my tool does"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input parameter",
                    "minLength": 1
                }
            },
            "required": ["input"],
            "additionalProperties": false
        })
    }

    fn execute(
        &self,
        args: Option<Value>,
        user: AuthenticatedUser,
    ) -> PinBoxedFuture<Result<Value, Error>> {
        Box::pin(async move {
            // Validate arguments
            validate_tool_args(&self.parameters_schema(), &args)?;

            // Extract arguments
            let args_obj = args.unwrap();
            let input = args_obj["input"].as_str().unwrap();

            // Access user's external credentials if needed
            if let Some(api_key) = user.get_external_key("some_service_key") {
                // Use the API key
            }

            // Execute tool logic
            let result = format!("Processed: {}", input);

            Ok(json!({ "output": result }))
        })
    }
}
```

### 2. Register the Tool

In `src/tools/mod.rs`:

```rust
// Add module declaration
pub mod my_tool;

// In initialize_all_tools() function, register your tool:
register_tool(
    my_tool::MyTool,
    &mut func_registry,
    &mut tool_definitions,
);
```

### 3. Test Your Tool

```bash
cargo build
cargo run
```

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "method": "invoke",
    "params": {
      "tool_name": "my_tool",
      "arguments": {
        "input": "test"
      }
    }
  }'
```

## Development

### Running in Development

```bash
cargo run
```

### Building for Production

```bash
cargo build --release
./target/release/mcp-server
```

### Testing

The project includes a comprehensive test suite with 160+ tests achieving industry-standard coverage targets.

**Run all tests:**

```bash
# Run tests sequentially (recommended for env var tests)
cargo test -- --test-threads=1

# Run tests in parallel (faster, but may have race conditions)
cargo test
```

**Run specific test suites:**

```bash
cargo test --test auth_tests          # Auth module tests
cargo test --test integration_tests   # Full HTTP integration tests
cargo test --test tools_proptest      # Property-based tests
```

**Check code coverage:**

```bash
# Install tarpaulin (one-time)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open coverage/index.html in your browser
```

### Test Coverage Guidelines

When adding new code, maintain these coverage targets:

| Module | Target | Reason |
|--------|--------|--------|
| `auth/` | **100%** | Security-critical authentication logic |
| `tools/` | **90%+** | Complex validation and execution paths |
| `lib.rs` | **80%+** | Core protocol handlers and routing |
| Overall | **85%+** | Industry standard for production code |

**Adding tests for new tools:**

When you create a new tool in `src/tools/`, add corresponding tests in `tests/tools_tests.rs`:

```rust
#[test]
fn test_my_tool_validates_required_params() {
    let schema = MyTool.parameters_schema();
    let args = None; // Missing required params

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
}

#[test]
fn test_my_tool_executes_successfully() {
    // Test happy path execution
}

#[test]
fn test_my_tool_handles_invalid_input() {
    // Test error cases
}
```

**Integration test pattern:**

Add integration tests in `tests/integration_tests.rs` for new endpoints:

```rust
#[tokio::test]
async fn test_my_tool_via_http() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header(
            http::HeaderName::from_static("authorization"),
            http::HeaderValue::from_str(&format!("Bearer {}", TEST_API_KEY)).unwrap()
        )
        .json(&json!({
            "method": "invoke",
            "params": {
                "tool_name": "my_tool",
                "arguments": {"input": "test"}
            }
        }))
        .await;

    response.assert_status_ok();
}
```

### Test Organization

```text
src/
└── main.rs                    # Server setup tests (2 tests)

tests/
├── common/mod.rs              # Shared test utilities
├── auth_tests.rs              # Auth module unit tests (18 tests)
├── auth_loader_tests.rs       # Credential loading tests (12 tests)
├── auth_middleware_tests.rs   # Middleware tests (14 tests)
├── tools_tests.rs             # Tool validation tests (51 tests)
├── tools_proptest.rs          # Property-based tests (13 tests)
├── lib_tests.rs               # Handler unit tests (34 tests)
└── integration_tests.rs       # Full HTTP tests (18 tests)
```

**Total: 162 tests** covering authentication, tool validation, protocol handling, and server initialization.

### Code Structure Guidelines

- **Modularity**: Keep auth, tools, and protocol logic separate
- **Error Handling**: Use `anyhow::Result` for flexible error propagation
- **Validation**: Always validate tool arguments using `validate_tool_args`
- **Security**: Never log API keys or sensitive credentials
- **Performance**: Use `Arc` for shared state, avoid unnecessary clones

## Security Considerations

1. **API Keys**: Store credentials in `config/credentials.toml` (not in git)
2. **HTTPS**: Use a reverse proxy (nginx, Caddy) for TLS in production
3. **Rate Limiting**: Consider adding rate limiting middleware
4. **Input Validation**: Always validate tool parameters using JSON Schema
5. **External Credentials**: Store per-user secrets in `external_keys`

## Troubleshooting

### Server won't start

- Check that `config/credentials.toml` exists and is valid TOML
- Verify port 3000 is not already in use
- Check file permissions on config directory

### Authentication failures

- Verify the `Authorization: Bearer <token>` header is correct
- Check that the API key exists in `credentials.toml`
- Ensure there are no whitespace issues in the API key

### Tool execution errors

- Check tool parameter validation in the schema
- Review error messages for specific validation failures
- Verify external credentials are configured if needed

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. **Add tests** - Required for all new features:
   - Unit tests for new functions/modules
   - Integration tests for new endpoints
   - Property tests for complex validation logic
5. **Verify coverage** meets targets:

   ```bash
   cargo test -- --test-threads=1  # All tests must pass
   cargo tarpaulin --out Html --output-dir coverage
   ```

6. Submit a pull request with:
   - Clear description of changes
   - Test coverage report summary
   - Any new dependencies justified

## Roadmap

- [ ] Add more example tools
- [ ] WebSocket support for streaming responses
- [ ] Unified src/tools auto-loader; potential macro development
- [ ] Rate limiting middleware
- [ ] Prometheus metrics
- [ ] Docker Compose setup
- [ ] Kubernetes deployment manifests
