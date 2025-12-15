use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub mod auth;
pub mod tools;

use auth::{AuthLayer, AuthenticatedUser, CredentialsStore};
use tools::{initialize_all_tools, ToolFunction};

// ============================================================================
// Error Codes (JSON-RPC 2.0)
// ============================================================================

pub const ERROR_AUTH: i32 = -32001;
pub const ERROR_INVALID_PARAMS: i32 = -32002;
pub const ERROR_TOOL_EXECUTION: i32 = -32003;
pub const ERROR_INVALID_REQUEST: i32 = -32600;
pub const ERROR_METHOD_NOT_FOUND: i32 = -32601;

// ============================================================================
// Request/Response Types
// ============================================================================

/// MCP request with method and params
#[derive(Debug, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    #[serde(rename = "discover")]
    Discover,
    #[serde(rename = "invoke")]
    Invoke {
        tool_name: String,
        arguments: Option<Value>,
    },
}

/// MCP response structure
#[derive(Debug, Serialize)]
pub struct McpResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>,
    pub jsonrpc: String,
}

impl McpResponse {
    /// Create success response
    pub fn success(result: Value) -> Self {
        Self {
            result: Some(result),
            error: None,
            jsonrpc: "2.0".to_string(),
        }
    }

    /// Create error response
    pub fn error(code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            result: None,
            error: Some(ErrorDetails {
                code,
                message,
                data,
            }),
            jsonrpc: "2.0".to_string(),
        }
    }
}

/// Error details for JSON-RPC responses
#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Tool definition for discovery
#[derive(Debug, Serialize, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
pub struct AppState {
    pub tool_registry: Arc<HashMap<String, ToolFunction>>,
    pub tool_definitions: Arc<Vec<ToolDefinition>>,
}

// ============================================================================
// Request Handler
// ============================================================================

/// Main MCP request handler
pub async fn handle_mcp_request(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<McpRequest>,
) -> Json<McpResponse> {
    match payload {
        McpRequest::Discover => {
            // Return list of all registered tools
            let tools_vec = (*state.tool_definitions).clone();
            Json(McpResponse::success(json!({ "tools": tools_vec })))
        }
        McpRequest::Invoke {
            tool_name,
            arguments,
        } => {
            // Lookup tool in registry
            if let Some(tool_func) = state.tool_registry.get(&tool_name) {
                // Execute tool
                match tool_func(arguments, user).await {
                    Ok(result) => Json(McpResponse::success(result)),
                    Err(e) => {
                        let error_msg = e.to_string();

                        // Classify error based on message content
                        let (error_code, error_prefix) = if is_param_validation_error(&error_msg)
                        {
                            (ERROR_INVALID_PARAMS, "Invalid parameters")
                        } else {
                            (ERROR_TOOL_EXECUTION, "Tool execution error")
                        };

                        Json(McpResponse::error(
                            error_code,
                            format!("{}: {}", error_prefix, error_msg),
                            None,
                        ))
                    }
                }
            } else {
                // Tool not found - return available tools
                let available_tools: Vec<String> = state
                    .tool_definitions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();

                Json(McpResponse::error(
                    ERROR_METHOD_NOT_FOUND,
                    format!("Tool '{}' not found", tool_name),
                    Some(json!({ "available_tools": available_tools })),
                ))
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Classify error as parameter validation error based on message keywords
pub fn is_param_validation_error(error_msg: &str) -> bool {
    let validation_keywords = [
        "parameter",
        "required",
        "Unexpected",
        "Missing",
        "must be",
        "exceeds maximum",
        "at least",
        "characters long",
        "type",
    ];

    validation_keywords
        .iter()
        .any(|keyword| error_msg.contains(keyword))
}

/// Health check endpoint
pub async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

// ============================================================================
// Application Factory
// ============================================================================

/// Create and configure the Axum application
pub fn create_app(credentials: CredentialsStore) -> Router {
    // Initialize tools
    let (func_registry, tool_definitions) = initialize_all_tools();

    let app_state = AppState {
        tool_registry: Arc::new(func_registry),
        tool_definitions: Arc::new(tool_definitions),
    };

    // Build router
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .with_state(app_state)
        .layer(AuthLayer::new(credentials))
        .route("/health", get(health_check))
}
