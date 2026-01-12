//! Tool schema generation for MCP.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Generate JSON schema for a type.
pub fn generate_schema<T: JsonSchema>() -> serde_json::Value {
    let schema = schemars::schema_for!(T);
    serde_json::to_value(schema).unwrap_or_default()
}

/// Tool definition with schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

impl ToolDefinition {
    /// Create a new tool definition.
    pub fn new(
        name: impl Into<String>,
        description: Option<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description,
            input_schema,
        }
    }

    /// Create a tool definition from a type with JsonSchema.
    pub fn from_type<T: JsonSchema>(name: impl Into<String>, description: Option<String>) -> Self {
        Self {
            name: name.into(),
            description,
            input_schema: generate_schema::<T>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(JsonSchema)]
    #[allow(dead_code)]
    struct TestInput {
        message: String,
        count: u32,
    }

    #[test]
    fn test_generate_schema() {
        let schema = generate_schema::<TestInput>();
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_tool_definition_from_type() {
        let tool =
            ToolDefinition::from_type::<TestInput>("test_tool", Some("A test tool".to_string()));
        assert_eq!(tool.name, "test_tool");
        assert!(tool.input_schema.get("properties").is_some());
    }
}
