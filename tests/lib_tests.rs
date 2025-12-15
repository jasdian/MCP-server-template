use axum::http::StatusCode;
use mcp_server::{
    AppState, ERROR_AUTH, ERROR_INVALID_PARAMS, ERROR_TOOL_EXECUTION, ErrorDetails, McpResponse,
    ToolDefinition, health_check, is_param_validation_error,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Error Classification Tests
// ============================================================================

#[test]
fn test_is_param_validation_error_with_parameter_keyword() {
    let error_msg = "Invalid parameter 'name'";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_required_keyword() {
    let error_msg = "Missing required field 'age'";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_missing_keyword() {
    let error_msg = "Missing value for key";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_must_be_keyword() {
    let error_msg = "Value must be greater than 10";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_unexpected_keyword() {
    let error_msg = "Unexpected parameter 'extra'";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_exceeds_maximum_keyword() {
    let error_msg = "String exceeds maximum length";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_at_least_keyword() {
    let error_msg = "Value must be at least 5";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_characters_long_keyword() {
    let error_msg = "String must be 10 characters long";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_type_keyword() {
    let error_msg = "Wrong type for field";
    assert!(is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_without_keywords() {
    let error_msg = "Database connection failed";
    assert!(!is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_with_empty_string() {
    let error_msg = "";
    assert!(!is_param_validation_error(error_msg));
}

#[test]
fn test_is_param_validation_error_case_sensitive() {
    // Keywords are case-sensitive
    let error_msg = "PARAMETER is invalid"; // "PARAMETER" not "parameter"
    assert!(!is_param_validation_error(error_msg));
}

// ============================================================================
// McpResponse Tests
// ============================================================================

#[test]
fn test_mcp_response_success_creates_result() {
    let result = json!({"foo": "bar"});
    let response = McpResponse::success(result.clone());

    assert_eq!(response.result, Some(result));
    assert!(response.error.is_none());
    assert_eq!(response.jsonrpc, "2.0");
}

#[test]
fn test_mcp_response_success_jsonrpc_version() {
    let response = McpResponse::success(json!({}));
    assert_eq!(response.jsonrpc, "2.0");
}

#[test]
fn test_mcp_response_error_creates_error() {
    let response = McpResponse::error(ERROR_INVALID_PARAMS, "Invalid parameter".to_string(), None);

    assert!(response.result.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.jsonrpc, "2.0");

    let error = response.error.unwrap();
    assert_eq!(error.code, ERROR_INVALID_PARAMS);
    assert_eq!(error.message, "Invalid parameter");
    assert!(error.data.is_none());
}

#[test]
fn test_mcp_response_error_with_data() {
    let data = json!({"details": "extra info"});
    let response = McpResponse::error(
        ERROR_TOOL_EXECUTION,
        "Execution failed".to_string(),
        Some(data.clone()),
    );

    let error = response.error.unwrap();
    assert_eq!(error.code, ERROR_TOOL_EXECUTION);
    assert_eq!(error.message, "Execution failed");
    assert_eq!(error.data, Some(data));
}

#[test]
fn test_mcp_response_error_different_codes() {
    let response1 = McpResponse::error(ERROR_AUTH, "Auth error".to_string(), None);
    let response2 = McpResponse::error(ERROR_INVALID_PARAMS, "Param error".to_string(), None);

    assert_eq!(response1.error.as_ref().unwrap().code, ERROR_AUTH);
    assert_eq!(response2.error.as_ref().unwrap().code, ERROR_INVALID_PARAMS);
}

// ============================================================================
// ErrorDetails Tests
// ============================================================================

#[test]
fn test_error_details_construction() {
    let details = ErrorDetails {
        code: -32002,
        message: "Test error".to_string(),
        data: Some(json!({"key": "value"})),
    };

    assert_eq!(details.code, -32002);
    assert_eq!(details.message, "Test error");
    assert!(details.data.is_some());
}

#[test]
fn test_error_details_without_data() {
    let details = ErrorDetails {
        code: -32001,
        message: "Simple error".to_string(),
        data: None,
    };

    assert_eq!(details.data, None);
}

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_health_check_returns_ok() {
    let (status, body) = health_check().await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "OK");
}

// ============================================================================
// AppState Tests
// ============================================================================

#[test]
fn test_app_state_construction() {
    let func_registry = HashMap::new();
    let tool_definitions = Vec::new();

    let state = AppState {
        tool_registry: Arc::new(func_registry),
        tool_definitions: Arc::new(tool_definitions),
    };

    assert_eq!(state.tool_registry.len(), 0);
    assert_eq!(state.tool_definitions.len(), 0);
}

#[test]
fn test_app_state_arc_wrapped() {
    let func_registry = HashMap::new();
    let tool_definitions = Vec::new();

    let state = AppState {
        tool_registry: Arc::new(func_registry),
        tool_definitions: Arc::new(tool_definitions),
    };

    // Should be able to clone cheaply (Arc increments reference count)
    let _state_clone = state.clone();
}

#[test]
fn test_app_state_with_definitions() {
    let func_registry = HashMap::new();

    let tool_definitions = vec![ToolDefinition {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        parameters: json!({}),
    }];

    let state = AppState {
        tool_registry: Arc::new(func_registry),
        tool_definitions: Arc::new(tool_definitions),
    };

    assert_eq!(state.tool_registry.len(), 0);
    assert_eq!(state.tool_definitions.len(), 1);
    assert_eq!(state.tool_definitions[0].name, "test_tool");
}

// ============================================================================
// ToolDefinition Tests
// ============================================================================

#[test]
fn test_tool_definition_construction() {
    let def = ToolDefinition {
        name: "my_tool".to_string(),
        description: "Does something useful".to_string(),
        parameters: json!({"type": "object"}),
    };

    assert_eq!(def.name, "my_tool");
    assert_eq!(def.description, "Does something useful");
    assert_eq!(def.parameters["type"], "object");
}

#[test]
fn test_tool_definition_clone() {
    let def = ToolDefinition {
        name: "tool".to_string(),
        description: "desc".to_string(),
        parameters: json!({}),
    };

    let cloned = def.clone();
    assert_eq!(def.name, cloned.name);
    assert_eq!(def.description, cloned.description);
}

// ============================================================================
// McpRequest Deserialization Tests
// ============================================================================

#[test]
fn test_mcp_request_discover_deserialization() {
    let json_str = r#"{"method": "discover"}"#;
    let result: Result<mcp_server::McpRequest, _> = serde_json::from_str(json_str);

    assert!(result.is_ok());
    match result.unwrap() {
        mcp_server::McpRequest::Discover => {} // Success
        _ => panic!("Expected Discover variant"),
    }
}

#[test]
fn test_mcp_request_invoke_deserialization() {
    let json_str = r#"{"method": "invoke", "params": {"tool_name": "test", "arguments": null}}"#;
    let result: Result<mcp_server::McpRequest, _> = serde_json::from_str(json_str);

    assert!(result.is_ok());
    match result.unwrap() {
        mcp_server::McpRequest::Invoke {
            tool_name,
            arguments,
        } => {
            assert_eq!(tool_name, "test");
            assert_eq!(arguments, None);
        }
        _ => panic!("Expected Invoke variant"),
    }
}

#[test]
fn test_mcp_request_invoke_with_arguments() {
    let json_str = r#"{"method": "invoke", "params": {"tool_name": "get_time", "arguments": {"timezone": "UTC"}}}"#;
    let result: Result<mcp_server::McpRequest, _> = serde_json::from_str(json_str);

    assert!(result.is_ok());
    match result.unwrap() {
        mcp_server::McpRequest::Invoke {
            tool_name,
            arguments,
        } => {
            assert_eq!(tool_name, "get_time");
            assert!(arguments.is_some());
            assert_eq!(arguments.unwrap()["timezone"], "UTC");
        }
        _ => panic!("Expected Invoke variant"),
    }
}

#[test]
fn test_mcp_request_invalid_json() {
    let json_str = r#"{"invalid": "json"}"#;
    let result: Result<mcp_server::McpRequest, _> = serde_json::from_str(json_str);

    assert!(result.is_err());
}

#[test]
fn test_mcp_request_missing_method() {
    let json_str = r#"{}"#;
    let result: Result<mcp_server::McpRequest, _> = serde_json::from_str(json_str);

    assert!(result.is_err());
}

// ============================================================================
// McpResponse Serialization Tests
// ============================================================================

#[test]
fn test_mcp_response_success_serialization() {
    let response = McpResponse::success(json!({"result": "data"}));
    let json_value = serde_json::to_value(&response).unwrap();

    assert_eq!(json_value["jsonrpc"], "2.0");
    assert_eq!(json_value["result"]["result"], "data");
    assert!(json_value["error"].is_null());
}

#[test]
fn test_mcp_response_error_serialization() {
    let response = McpResponse::error(
        ERROR_INVALID_PARAMS,
        "Bad params".to_string(),
        Some(json!({"detail": "info"})),
    );
    let json_value = serde_json::to_value(&response).unwrap();

    assert_eq!(json_value["jsonrpc"], "2.0");
    assert!(json_value["result"].is_null());
    assert_eq!(json_value["error"]["code"], ERROR_INVALID_PARAMS);
    assert_eq!(json_value["error"]["message"], "Bad params");
    assert_eq!(json_value["error"]["data"]["detail"], "info");
}

#[test]
fn test_mcp_response_error_no_data_omitted() {
    let response = McpResponse::error(ERROR_AUTH, "Unauthorized".to_string(), None);
    let json_value = serde_json::to_value(&response).unwrap();

    // data field should be omitted (not null, but missing)
    assert!(
        !json_value["error"]
            .as_object()
            .unwrap()
            .contains_key("data")
    );
}

// ============================================================================
// Error Code Constants Tests
// ============================================================================

#[test]
fn test_error_codes_are_correct() {
    assert_eq!(ERROR_AUTH, -32001);
    assert_eq!(ERROR_INVALID_PARAMS, -32002);
    assert_eq!(ERROR_TOOL_EXECUTION, -32003);
    assert_eq!(mcp_server::ERROR_INVALID_REQUEST, -32600);
    assert_eq!(mcp_server::ERROR_METHOD_NOT_FOUND, -32601);
}
