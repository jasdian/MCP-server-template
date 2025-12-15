mod common;

use common::*;
use mcp_server::{create_app, ERROR_AUTH, ERROR_METHOD_NOT_FOUND};
use axum_test::TestServer;
use serde_json::{json, Value};

// ============================================================================
// Health Endpoint Tests (No Auth Required)
// ============================================================================

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/health").await;

    response.assert_status_ok();
    response.assert_text("OK");
}

// ============================================================================
// Authentication Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_without_authorization_header() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .json(&json!({"method": "discover"}))
        .await;

    response.assert_status_unauthorized();

    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["error"]["code"], ERROR_AUTH);
    assert!(body["error"]["message"].as_str().unwrap().contains("Missing Authorization header"));
}

#[tokio::test]
async fn test_mcp_with_invalid_token() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", "Bearer invalid-token-xyz")
        .json(&json!({"method": "discover"}))
        .await;

    response.assert_status_unauthorized();

    let body: Value = response.json();
    assert_eq!(body["error"]["code"], ERROR_AUTH);
    assert!(body["error"]["message"].as_str().unwrap().contains("Invalid or expired API key"));
}

#[tokio::test]
async fn test_mcp_with_malformed_auth_header() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", "Basic dGVzdDp0ZXN0")
        .json(&json!({"method": "discover"}))
        .await;

    response.assert_status_unauthorized();

    let body: Value = response.json();
    assert_eq!(body["error"]["code"], ERROR_AUTH);
}

#[tokio::test]
async fn test_mcp_with_valid_token_proceeds() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"].is_object());
}

// ============================================================================
// Discover Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_discover_returns_tools_list() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"]["tools"].is_array());
}

#[tokio::test]
async fn test_discover_includes_get_current_time() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    let body: Value = response.json();
    let tools = body["result"]["tools"].as_array().unwrap();

    let get_time_tool = tools.iter().find(|t| t["name"] == "get_current_time");
    assert!(get_time_tool.is_some());

    let tool = get_time_tool.unwrap();
    assert!(!tool["description"].as_str().unwrap().is_empty());
    assert!(tool["parameters"].is_object());
}

#[tokio::test]
async fn test_discover_tool_definition_structure() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    let body: Value = response.json();
    let tools = body["result"]["tools"].as_array().unwrap();

    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        assert!(tool["parameters"].is_object());
    }
}

// ============================================================================
// Invoke Endpoint - Success Cases
// ============================================================================

#[tokio::test]
async fn test_invoke_get_current_time_success() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({
            "method": "invoke",
            "params": {
                "tool_name": "get_current_time",
                "arguments": null
            }
        }))
        .await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"]["current_time"].is_string());

    // Verify ISO 8601 format (basic check)
    let time_str = body["result"]["current_time"].as_str().unwrap();
    assert!(time_str.contains('T'));
    assert!(time_str.contains('Z') || time_str.contains('+') || time_str.contains('-'));
}

#[tokio::test]
async fn test_invoke_returns_proper_json_rpc_response() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({
            "method": "invoke",
            "params": {
                "tool_name": "get_current_time",
                "arguments": null
            }
        }))
        .await;

    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"].is_object());
    assert!(body["error"].is_null());
}

// ============================================================================
// Invoke Endpoint - Error Cases
// ============================================================================

#[tokio::test]
async fn test_invoke_nonexistent_tool() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({
            "method": "invoke",
            "params": {
                "tool_name": "nonexistent_tool",
                "arguments": null
            }
        }))
        .await;

    response.assert_status_ok(); // Still 200, but with JSON-RPC error

    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["error"]["code"], ERROR_METHOD_NOT_FOUND);
    assert!(body["error"]["message"].as_str().unwrap().contains("not found"));
    assert!(body["error"]["data"]["available_tools"].is_array());
}

#[tokio::test]
async fn test_invoke_nonexistent_tool_includes_available_tools() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({
            "method": "invoke",
            "params": {
                "tool_name": "unknown_tool",
                "arguments": null
            }
        }))
        .await;

    let body: Value = response.json();
    let available_tools = body["error"]["data"]["available_tools"].as_array().unwrap();

    assert!(!available_tools.is_empty());
    assert!(available_tools.iter().any(|t| t == "get_current_time"));
}

// ============================================================================
// End-to-End Flows
// ============================================================================

#[tokio::test]
async fn test_full_discovery_then_invoke_flow() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    // First, discover tools
    let discover_response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    discover_response.assert_status_ok();
    let discover_body: Value = discover_response.json();
    let tools = discover_body["result"]["tools"].as_array().unwrap();
    let tool_name = tools[0]["name"].as_str().unwrap();

    // Then, invoke the discovered tool
    let invoke_response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({
            "method": "invoke",
            "params": {
                "tool_name": tool_name,
                "arguments": null
            }
        }))
        .await;

    invoke_response.assert_status_ok();
    let invoke_body: Value = invoke_response.json();
    assert!(invoke_body["result"].is_object());
}

#[tokio::test]
async fn test_multiple_requests_same_credentials() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    // First request
    let response1 = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    response1.assert_status_ok();

    // Second request
    let response2 = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({
            "method": "invoke",
            "params": {
                "tool_name": "get_current_time",
                "arguments": null
            }
        }))
        .await;

    response2.assert_status_ok();
}

#[tokio::test]
async fn test_requests_from_different_users() {
    let credentials = create_multi_user_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    // Request from first user
    let response1 = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    response1.assert_status_ok();

    // Request from second user
    let response2 = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY_2))
        .json(&json!({"method": "discover"}))
        .await;

    response2.assert_status_ok();
}

// ============================================================================
// Response Validation Tests
// ============================================================================

#[tokio::test]
async fn test_all_responses_have_jsonrpc_2_0() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    // Test discover
    let response1 = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    let body1: Value = response1.json();
    assert_eq!(body1["jsonrpc"], "2.0");

    // Test invoke
    let response2 = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({
            "method": "invoke",
            "params": {
                "tool_name": "get_current_time",
                "arguments": null
            }
        }))
        .await;

    let body2: Value = response2.json();
    assert_eq!(body2["jsonrpc"], "2.0");

    // Test error
    let response3 = server
        .post("/mcp")
        .add_header("Authorization", "Bearer invalid")
        .json(&json!({"method": "discover"}))
        .await;

    let body3: Value = response3.json();
    assert_eq!(body3["jsonrpc"], "2.0");
}

#[tokio::test]
async fn test_error_responses_have_proper_structure() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .json(&json!({"method": "discover"}))
        .await;

    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"].is_null());
    assert!(body["error"].is_object());
    assert!(body["error"]["code"].is_number());
    assert!(body["error"]["message"].is_string());
}

#[tokio::test]
async fn test_success_responses_have_proper_structure() {
    let credentials = create_test_credentials_store();
    let app = create_app(credentials);
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/mcp")
        .add_header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .json(&json!({"method": "discover"}))
        .await;

    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"].is_object());
    assert!(body["error"].is_null());
}
