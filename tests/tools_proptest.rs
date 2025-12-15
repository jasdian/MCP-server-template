mod common;

use mcp_server::tools::validate_tool_args;
use proptest::prelude::*;
use serde_json::json;

// ============================================================================
// Property-Based Tests for Tool Validation
// ============================================================================

proptest! {
    /// Any valid string should pass basic string type validation
    #[test]
    fn prop_valid_string_always_validates(s in ".*") {
        let schema = json!({
            "type": "object",
            "properties": {
                "text": {"type": "string"}
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"text": s}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_ok());
    }

    /// String length constraints should work with random lengths
    #[test]
    fn prop_string_length_constraints_work(
        s in prop::string::string_regex("[a-z]{5,10}").unwrap()
    ) {
        // String is 5-10 chars, so should pass minLength: 3, maxLength: 20
        let schema = json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "minLength": 3,
                    "maxLength": 20
                }
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"text": s}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_ok());
    }

    /// String shorter than minLength should fail
    #[test]
    fn prop_string_too_short_fails(
        s in prop::string::string_regex("[a-z]{1,2}").unwrap()
    ) {
        let schema = json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "minLength": 5
                }
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"text": s}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_err());
    }

    /// String longer than maxLength should fail
    #[test]
    fn prop_string_too_long_fails(
        s in prop::string::string_regex("[a-z]{15,30}").unwrap()
    ) {
        let schema = json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "maxLength": 10
                }
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"text": s}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_err());
    }

    /// Numeric constraints should work with random values
    #[test]
    fn prop_numeric_constraints_work(n in 10.0..100.0_f64) {
        let schema = json!({
            "type": "object",
            "properties": {
                "value": {
                    "type": "number",
                    "minimum": 5.0,
                    "maximum": 150.0
                }
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"value": n}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_ok());
    }

    /// Number below minimum should fail
    #[test]
    fn prop_number_below_minimum_fails(n in -100.0..0.0_f64) {
        let schema = json!({
            "type": "object",
            "properties": {
                "value": {
                    "type": "number",
                    "minimum": 10.0
                }
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"value": n}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_err());
    }

    /// Number above maximum should fail
    #[test]
    fn prop_number_above_maximum_fails(n in 200.0..1000.0_f64) {
        let schema = json!({
            "type": "object",
            "properties": {
                "value": {
                    "type": "number",
                    "maximum": 100.0
                }
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"value": n}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_err());
    }

    /// Type validation should reject wrong types (integers)
    #[test]
    fn prop_type_validation_rejects_integers_for_string(n in any::<i64>()) {
        // Expect string but provide integer
        let schema = json!({
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"field": n}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_err());
    }

    /// Type validation should reject wrong types (booleans)
    #[test]
    fn prop_type_validation_rejects_booleans_for_string(b in any::<bool>()) {
        // Expect string but provide boolean
        let schema = json!({
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"field": b}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_err());
    }

    /// Additional properties false should reject random extra string keys
    #[test]
    fn prop_additional_properties_false_rejects_extra_keys(
        extra_key in "[a-z]{3,10}",
        extra_value in any::<String>()
    ) {
        // Only "name" is allowed
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": [],
            "additionalProperties": false
        });

        // Add random extra field
        let args = Some(json!({
            "name": "Alice",
            extra_key.clone(): extra_value
        }));

        let result = validate_tool_args(&schema, &args);
        // Should fail unless extra_key happens to be "name" (unlikely)
        if extra_key != "name" {
            assert!(result.is_err());
        }
    }

    /// Integer values should work for both integer and number types
    #[test]
    fn prop_integer_works_for_number_type(n in any::<i64>()) {
        let schema = json!({
            "type": "object",
            "properties": {
                "value": {"type": "number"}
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"value": n}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_ok());
    }

    /// Array maxItems constraint with random arrays
    #[test]
    fn prop_array_max_items_constraint(
        arr in prop::collection::vec(any::<i32>(), 0..=5)
    ) {
        // Array size 0-5, maxItems is 10, so should pass
        let schema = json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "maxItems": 10
                }
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"items": arr}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_ok());
    }

    /// Array exceeding maxItems should fail
    #[test]
    fn prop_array_exceeding_max_items_fails(
        arr in prop::collection::vec(any::<i32>(), 6..=15)
    ) {
        // Array size 6-15, maxItems is 5, so should fail
        let schema = json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "maxItems": 5
                }
            },
            "required": [],
            "additionalProperties": false
        });
        let args = Some(json!({"items": arr}));

        let result = validate_tool_args(&schema, &args);
        assert!(result.is_err());
    }
}
