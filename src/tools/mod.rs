use crate::ToolDefinition;
use crate::auth::AuthenticatedUser;
use anyhow::{Error, Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

// Re-export the macro for convenience
pub use mcp_server_macros::mcp_tool;

pub mod get_time;

pub type PinBoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;
pub type ToolFunction = Box<
    dyn Fn(Option<Value>, AuthenticatedUser) -> PinBoxedFuture<Result<Value, Error>> + Send + Sync,
>;

/// Trait for MCP tools
/// All tools must implement this trait to be registered
pub trait McpTool {
    /// Tool name (must be unique)
    fn name(&self) -> &'static str;

    /// Human-readable description
    fn description(&self) -> &'static str;

    /// JSON Schema for parameters
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with given arguments and authenticated user
    fn execute(
        &self,
        args: Option<Value>,
        user: AuthenticatedUser,
    ) -> PinBoxedFuture<Result<Value, Error>>;
}

/// Helper trait for tool registration (used by the #[mcp_tool] macro)
pub trait ToolRegistration {
    fn tool_instance() -> Box<dyn McpTool + Send + Sync>;
}

/// Entry in the inventory for tool collection
pub struct ToolEntry {
    pub constructor: fn() -> Box<dyn McpTool + Send + Sync>,
}

// Collect all tools annotated with #[mcp_tool]
inventory::collect!(ToolEntry);

/// Validate tool arguments against JSON Schema
pub fn validate_tool_args(schema: &Value, args: &Option<Value>) -> Result<()> {
    let properties = schema.get("properties").and_then(|p| p.as_object());
    let required = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    let has_required = !required.is_empty();
    let additional_properties = schema
        .get("additionalProperties")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if has_required && args.is_none() {
        return Err(anyhow!(
            "Missing required arguments: {}",
            required.join(", ")
        ));
    }

    if args.is_none() {
        return Ok(());
    }

    let args_obj = args
        .as_ref()
        .and_then(|a| a.as_object())
        .ok_or_else(|| anyhow!("Arguments must be an object"))?;

    if !additional_properties && let Some(props) = properties {
        for key in args_obj.keys() {
            if !props.contains_key(key) {
                return Err(anyhow!("Unexpected parameter: '{}'", key));
            }
        }
    }

    for req_field in &required {
        if !args_obj.contains_key(*req_field) {
            return Err(anyhow!("Missing required parameter: '{}'", req_field));
        }
    }

    if let Some(props) = properties {
        for (prop_name, value) in args_obj {
            if let Some(prop_schema) = props.get(prop_name) {
                validate_value(prop_name, value, prop_schema)?;
            }
        }
    }

    Ok(())
}

/// Validate a single value against its schema
fn validate_value(name: &str, value: &Value, schema: &Value) -> Result<()> {
    if let Some(expected_type) = schema.get("type").and_then(|t| t.as_str()) {
        let actual_type = match value {
            Value::String(_) => "string",
            Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Null => "null",
        };

        let type_matches =
            expected_type == actual_type || (expected_type == "number" && actual_type == "integer");

        if !type_matches {
            return Err(anyhow!(
                "Parameter '{}' must be of type '{}', got '{}'",
                name,
                expected_type,
                actual_type
            ));
        }
    }

    if let Some(s) = value.as_str() {
        if let Some(min_len) = schema.get("minLength").and_then(|v| v.as_u64())
            && s.len() < min_len as usize
        {
            return Err(anyhow!(
                "Parameter '{}' must be at least {} characters long",
                name,
                min_len
            ));
        }

        if let Some(max_len) = schema.get("maxLength").and_then(|v| v.as_u64())
            && s.len() > max_len as usize
        {
            return Err(anyhow!(
                "Parameter '{}' exceeds maximum length of {}",
                name,
                max_len
            ));
        }

        if let Some(pattern) = schema.get("pattern").and_then(|v| v.as_str()) {
            // Basic pattern matching (prefix matching for simplicity)
            if pattern.starts_with('^') && pattern.ends_with('*') {
                let prefix = pattern.trim_start_matches('^').trim_end_matches('*');
                if !s.starts_with(prefix) {
                    return Err(anyhow!(
                        "Parameter '{}' does not match required pattern",
                        name
                    ));
                }
            }
        }
    }

    if let Some(n) = value.as_f64() {
        if let Some(min) = schema.get("minimum").and_then(|v| v.as_f64())
            && n < min
        {
            return Err(anyhow!("Parameter '{}' must be at least {}", name, min));
        }

        if let Some(max) = schema.get("maximum").and_then(|v| v.as_f64())
            && n > max
        {
            return Err(anyhow!("Parameter '{}' must be at most {}", name, max));
        }
    }

    if let Some(arr) = value.as_array()
        && let Some(max_items) = schema.get("maxItems").and_then(|v| v.as_u64())
        && arr.len() > max_items as usize
    {
        return Err(anyhow!(
            "Parameter '{}' exceeds maximum array length of {}",
            name,
            max_items
        ));
    }

    Ok(())
}

/// Initialize all tools and return registry and definitions
/// Tools are automatically discovered via the inventory system
pub fn initialize_all_tools() -> (HashMap<String, ToolFunction>, Vec<ToolDefinition>) {
    let mut func_registry = HashMap::new();
    let mut tool_definitions = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    // Auto-discover all tools annotated with #[mcp_tool]
    for entry in inventory::iter::<ToolEntry> {
        let tool = (entry.constructor)();
        let name = tool.name();

        // Check for duplicate tool names
        if !seen_names.insert(name.to_string()) {
            panic!(
                "Duplicate tool name detected: '{}'. Each tool must have a unique name.",
                name
            );
        }

        register_tool_boxed(tool, &mut func_registry, &mut tool_definitions);
    }

    (func_registry, tool_definitions)
}

/// Register a boxed tool instance (used internally for auto-registration)
fn register_tool_boxed(
    tool: Box<dyn McpTool + Send + Sync>,
    func_reg: &mut HashMap<String, ToolFunction>,
    def_vec: &mut Vec<ToolDefinition>,
) {
    let name = tool.name().to_string();

    // Add to definitions (for discover endpoint)
    def_vec.push(ToolDefinition {
        name: name.clone(),
        description: tool.description().to_string(),
        parameters: tool.parameters_schema(),
    });

    // Add to function registry (for invoke endpoint)
    let tool_arc: std::sync::Arc<dyn McpTool + Send + Sync> = std::sync::Arc::from(tool);
    let execution_closure =
        move |args: Option<Value>, user: AuthenticatedUser| tool_arc.execute(args, user);

    func_reg.insert(name, Box::new(execution_closure));
}
