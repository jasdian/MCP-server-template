mod common;

use common::*;
use mcp_server::tools::{initialize_all_tools, register_tool, validate_tool_args};
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// Type Validation Tests
// ============================================================================

#[test]
fn test_validate_string_type_valid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"name": "test"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_validate_string_type_invalid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"name": 123}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be of type 'string'"));
    assert!(err_msg.contains("got 'integer'"));
}

#[test]
fn test_validate_integer_type_valid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "count": {"type": "integer"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"count": 42}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_validate_integer_type_invalid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "count": {"type": "integer"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"count": "not a number"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
}

#[test]
fn test_validate_number_type_accepts_integer() {
    let schema = json!({
        "type": "object",
        "properties": {
            "value": {"type": "number"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"value": 42}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_validate_number_type_accepts_float() {
    let schema = json!({
        "type": "object",
        "properties": {
            "value": {"type": "number"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"value": 4.2}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_validate_boolean_type_valid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "enabled": {"type": "boolean"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"enabled": true}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_validate_boolean_type_invalid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "enabled": {"type": "boolean"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"enabled": "yes"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
}

#[test]
fn test_validate_array_type_valid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "items": {"type": "array"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"items": [1, 2, 3]}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_validate_array_type_invalid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "items": {"type": "array"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"items": "not an array"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
}

#[test]
fn test_validate_object_type_valid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "config": {"type": "object"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"config": {"key": "value"}}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_validate_object_type_invalid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "config": {"type": "object"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"config": "not an object"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
}

#[test]
fn test_validate_null_type_valid() {
    let schema = json!({
        "type": "object",
        "properties": {
            "optional": {"type": "null"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"optional": null}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

// ============================================================================
// Required Fields Tests
// ============================================================================

#[test]
fn test_missing_required_field() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"}
        },
        "required": ["name"],
        "additionalProperties": false
    });
    let args = Some(json!({}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Missing required parameter: 'name'"));
}

#[test]
fn test_all_required_fields_present() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        },
        "required": ["name", "age"],
        "additionalProperties": false
    });
    let args = Some(json!({"name": "Alice", "age": 30}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_no_required_fields() {
    let schema = json!({
        "type": "object",
        "properties": {
            "optional": {"type": "string"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_missing_required_with_none_args() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"}
        },
        "required": ["name"],
        "additionalProperties": false
    });
    let args = None;

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Missing required arguments"));
}

// ============================================================================
// Additional Properties Tests
// ============================================================================

#[test]
fn test_additional_properties_false_with_extra_field() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"name": "Alice", "age": 30}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unexpected parameter: 'age'"));
}

#[test]
fn test_additional_properties_false_with_only_expected_fields() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"name": "Alice", "age": 30}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_additional_properties_true_with_extra_field() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"}
        },
        "required": [],
        "additionalProperties": true
    });
    let args = Some(json!({"name": "Alice", "extra": "allowed"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

// ============================================================================
// String Constraints Tests
// ============================================================================

#[test]
fn test_string_min_length_pass() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "minLength": 3}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"name": "Alice"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_string_min_length_fail() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "minLength": 5}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"name": "Bob"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be at least 5 characters long"));
}

#[test]
fn test_string_max_length_pass() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "maxLength": 10}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"name": "Alice"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_string_max_length_fail() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "maxLength": 5}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"name": "Alexander"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("exceeds maximum length of 5"));
}

#[test]
fn test_string_pattern_prefix_match() {
    let schema = json!({
        "type": "object",
        "properties": {
            "code": {"type": "string", "pattern": "^USER*"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"code": "USER123"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_string_pattern_prefix_no_match() {
    let schema = json!({
        "type": "object",
        "properties": {
            "code": {"type": "string", "pattern": "^USER*"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"code": "ADMIN123"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("does not match required pattern"));
}

#[test]
fn test_string_combined_constraints() {
    let schema = json!({
        "type": "object",
        "properties": {
            "username": {
                "type": "string",
                "minLength": 3,
                "maxLength": 20
            }
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"username": "alice"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

// ============================================================================
// Numeric Constraints Tests
// ============================================================================

#[test]
fn test_number_minimum_pass() {
    let schema = json!({
        "type": "object",
        "properties": {
            "age": {"type": "integer", "minimum": 18}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"age": 25}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_number_minimum_fail() {
    let schema = json!({
        "type": "object",
        "properties": {
            "age": {"type": "integer", "minimum": 18}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"age": 15}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be at least 18"));
}

#[test]
fn test_number_maximum_pass() {
    let schema = json!({
        "type": "object",
        "properties": {
            "age": {"type": "integer", "maximum": 100}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"age": 75}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_number_maximum_fail() {
    let schema = json!({
        "type": "object",
        "properties": {
            "age": {"type": "integer", "maximum": 100}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"age": 150}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be at most 100"));
}

#[test]
fn test_number_combined_constraints() {
    let schema = json!({
        "type": "object",
        "properties": {
            "score": {
                "type": "number",
                "minimum": 0,
                "maximum": 100
            }
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"score": 85.5}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_number_constraints_work_for_integers() {
    let schema = json!({
        "type": "object",
        "properties": {
            "count": {
                "type": "integer",
                "minimum": 1,
                "maximum": 10
            }
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"count": 5}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

// ============================================================================
// Array Constraints Tests
// ============================================================================

#[test]
fn test_array_max_items_pass() {
    let schema = json!({
        "type": "object",
        "properties": {
            "tags": {"type": "array", "maxItems": 5}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"tags": [1, 2, 3]}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_array_max_items_fail() {
    let schema = json!({
        "type": "object",
        "properties": {
            "tags": {"type": "array", "maxItems": 3}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"tags": [1, 2, 3, 4, 5]}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("exceeds maximum array length of 3"));
}

#[test]
fn test_array_empty() {
    let schema = json!({
        "type": "object",
        "properties": {
            "tags": {"type": "array"}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"tags": []}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_array_at_limit() {
    let schema = json!({
        "type": "object",
        "properties": {
            "tags": {"type": "array", "maxItems": 3}
        },
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!({"tags": [1, 2, 3]}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_empty_schema() {
    let schema = json!({
        "type": "object",
        "properties": {},
        "required": [],
        "additionalProperties": true
    });
    let args = Some(json!({"anything": "goes"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_no_properties_defined() {
    let schema = json!({
        "type": "object",
        "required": [],
        "additionalProperties": true
    });
    let args = Some(json!({"field": "value"}));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_null_arguments_no_requirements() {
    let schema = json!({
        "type": "object",
        "properties": {},
        "required": [],
        "additionalProperties": false
    });
    let args = None;

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_ok());
}

#[test]
fn test_arguments_not_object() {
    let schema = json!({
        "type": "object",
        "properties": {},
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!("not an object"));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Arguments must be an object"));
}

#[test]
fn test_arguments_array_not_object() {
    let schema = json!({
        "type": "object",
        "properties": {},
        "required": [],
        "additionalProperties": false
    });
    let args = Some(json!([1, 2, 3]));

    let result = validate_tool_args(&schema, &args);
    assert!(result.is_err());
}

// ============================================================================
// register_tool Tests
// ============================================================================

#[test]
fn test_register_tool_adds_to_registry() {
    let mut func_registry = HashMap::new();
    let mut tool_definitions = Vec::new();
    let tool = MockTool::new("test_tool", "A test tool");

    register_tool(tool, &mut func_registry, &mut tool_definitions);

    assert!(func_registry.contains_key("test_tool"));
}

#[test]
fn test_register_tool_adds_to_definitions() {
    let mut func_registry = HashMap::new();
    let mut tool_definitions = Vec::new();
    let tool = MockTool::new("test_tool", "A test tool");

    register_tool(tool, &mut func_registry, &mut tool_definitions);

    assert_eq!(tool_definitions.len(), 1);
    assert_eq!(tool_definitions[0].name, "test_tool");
}

#[test]
fn test_register_tool_name_matches() {
    let mut func_registry = HashMap::new();
    let mut tool_definitions = Vec::new();
    let tool = MockTool::new("my_tool", "My test tool");

    register_tool(tool, &mut func_registry, &mut tool_definitions);

    assert_eq!(tool_definitions[0].name, "my_tool");
    assert!(func_registry.contains_key("my_tool"));
}

#[test]
fn test_register_tool_description_matches() {
    let mut func_registry = HashMap::new();
    let mut tool_definitions = Vec::new();
    let tool = MockTool::new("test_tool", "This is a test tool");

    register_tool(tool, &mut func_registry, &mut tool_definitions);

    assert_eq!(tool_definitions[0].description, "This is a test tool");
}

#[test]
fn test_register_tool_schema_matches() {
    let mut func_registry = HashMap::new();
    let mut tool_definitions = Vec::new();
    let tool = MockTool::new("test_tool", "A test tool");

    register_tool(tool, &mut func_registry, &mut tool_definitions);

    let schema = &tool_definitions[0].parameters;
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["test_param"].is_object());
}

// ============================================================================
// initialize_all_tools Tests
// ============================================================================

#[test]
fn test_initialize_all_tools_returns_hashmap_and_vec() {
    let (func_registry, tool_definitions) = initialize_all_tools();

    assert!(!func_registry.is_empty());
    assert!(!tool_definitions.is_empty());
}

#[test]
fn test_initialize_all_tools_includes_get_time_tool() {
    let (func_registry, tool_definitions) = initialize_all_tools();

    assert!(func_registry.contains_key("get_current_time"));
    assert!(tool_definitions.iter().any(|def| def.name == "get_current_time"));
}

#[test]
fn test_initialize_all_tools_registry_and_definitions_match() {
    let (func_registry, tool_definitions) = initialize_all_tools();

    assert_eq!(func_registry.len(), tool_definitions.len());

    for def in &tool_definitions {
        assert!(
            func_registry.contains_key(&def.name),
            "Registry should contain tool: {}",
            def.name
        );
    }
}

#[test]
fn test_initialize_all_tools_get_time_definition() {
    let (_func_registry, tool_definitions) = initialize_all_tools();

    let get_time_def = tool_definitions
        .iter()
        .find(|def| def.name == "get_current_time")
        .expect("GetTimeTool should be registered");

    assert!(!get_time_def.description.is_empty());
    assert!(get_time_def.parameters.is_object());
}
