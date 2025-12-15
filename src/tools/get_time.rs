use super::{mcp_tool, McpTool, PinBoxedFuture, validate_tool_args};
use crate::auth::AuthenticatedUser;
use anyhow::{Error, Result};
use chrono::Utc;
use serde_json::{Value, json};

/// Simple tool that returns the current server time
#[mcp_tool]
pub struct GetTimeTool;

impl McpTool for GetTimeTool {
    fn name(&self) -> &'static str {
        "get_current_time"
    }

    fn description(&self) -> &'static str {
        "Returns the current server time as an ISO 8601 string."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false,
            "required": []
        })
    }

    fn execute(
        &self,
        args: Option<Value>,
        _user: AuthenticatedUser,
    ) -> PinBoxedFuture<Result<Value, Error>> {
        let schema = self.parameters_schema();

        Box::pin(async move {
            validate_tool_args(&schema, &args)?;

            // ISO 8601 format
            let current_time = Utc::now().to_rfc3339();

            Ok(json!({
                "current_time": current_time
            }))
        })
    }
}
