//! Tool definition and handler trait for MCP tools.

use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

use crate::error::McpSdkError;

/// Trait for implementing MCP tool handlers.
///
/// Implement this trait to define custom tool behavior. The handler receives
/// tool input as a JSON [`Value`] and returns a JSON [`Value`] on success.
///
/// # Example
///
/// ```ignore
/// use claude_agent_mcp_sdk::tool::ToolHandler;
/// use claude_agent_mcp_sdk::error::McpSdkError;
/// use serde_json::Value;
///
/// struct GreetingHandler;
///
/// impl ToolHandler for GreetingHandler {
///     fn call(
///         &self,
///         input: Value,
///     ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
///         Box::pin(async move {
///             let name = input.get("name")
///                 .and_then(|v| v.as_str())
///                 .unwrap_or("World");
///             Ok(serde_json::json!({
///                 "content": [{ "type": "text", "text": format!("Hello, {}!", name) }]
///             }))
///         })
///     }
/// }
/// ```
pub trait ToolHandler: Send + Sync {
    /// Invoke the tool with the given input.
    fn call(
        &self,
        input: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>>;
}

/// A complete definition of an MCP tool, including its metadata and handler.
pub struct ToolDefinition {
    /// The tool name, used as the identifier in MCP protocol messages.
    pub name: String,
    /// A human-readable description of what the tool does.
    pub description: String,
    /// JSON Schema describing the tool's expected input parameters.
    pub input_schema: Value,
    /// The handler that executes when the tool is called.
    pub handler: Box<dyn ToolHandler>,
}

impl ToolDefinition {
    /// Create a new tool definition with the given components.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        handler: impl ToolHandler + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            handler: Box::new(handler),
        }
    }
}

/// Ergonomic constructor for creating a [`ToolDefinition`].
///
/// Similar to Python's `@tool` decorator, this function provides a concise
/// way to define a custom MCP tool.
///
/// # Example
///
/// ```ignore
/// use claude_agent_mcp_sdk::tool;
///
/// let echo_tool = tool(
///     "echo",
///     "Repeats the input message",
///     serde_json::json!({
///         "type": "object",
///         "properties": {
///             "message": { "type": "string", "description": "Message to echo" }
///         },
///         "required": ["message"]
///     }),
///     |input| Box::pin(async move {
///         let message = input.get("message").and_then(|v| v.as_str()).unwrap_or("");
///         Ok(serde_json::json!({
///             "content": [{ "type": "text", "text": message }]
///         }))
///     }),
/// );
/// ```
pub fn tool(
    name: impl Into<String>,
    description: impl Into<String>,
    input_schema: Value,
    handler: impl ToolHandler + 'static,
) -> ToolDefinition {
    ToolDefinition::new(name, description, input_schema, handler)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AddHandler;

    impl ToolHandler for AddHandler {
        fn call(
            &self,
            input: Value,
        ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
            Box::pin(async move {
                let a = input.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = input.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": format!("{}", a + b) }]
                }))
            })
        }
    }

    #[tokio::test]
    async fn test_tool_handler_trait() {
        let handler = AddHandler;
        let input = serde_json::json!({ "a": 2.0, "b": 3.0 });
        let result = handler.call(input).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "5");
    }

    #[test]
    fn test_tool_definition_new() {
        let def = ToolDefinition::new(
            "test",
            "A test tool",
            serde_json::json!({ "type": "object" }),
            AddHandler,
        );
        assert_eq!(def.name, "test");
        assert_eq!(def.description, "A test tool");
    }

    #[test]
    fn test_tool_function() {
        let def =
            tool("greet", "Greets the user", serde_json::json!({ "type": "object" }), AddHandler);
        assert_eq!(def.name, "greet");
        assert_eq!(def.description, "Greets the user");
    }

    #[tokio::test]
    async fn test_tool_definition_handler_invocation() {
        let def = ToolDefinition::new(
            "add",
            "Adds two numbers",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                }
            }),
            AddHandler,
        );

        let input = serde_json::json!({ "a": 10.0, "b": 20.0 });
        let result = def.handler.call(input).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "30");
    }

    #[tokio::test]
    async fn test_tool_function_with_closure_handler() {
        struct ClosureHandler;
        impl ToolHandler for ClosureHandler {
            fn call(
                &self,
                input: Value,
            ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
                Box::pin(async move {
                    let msg = input.get("message").and_then(|v| v.as_str()).unwrap_or("");
                    Ok(serde_json::json!({
                        "content": [{ "type": "text", "text": msg }]
                    }))
                })
            }
        }

        let def = tool(
            "echo",
            "Echoes the message",
            serde_json::json!({
                "type": "object",
                "properties": { "message": { "type": "string" } }
            }),
            ClosureHandler,
        );

        let input = serde_json::json!({ "message": "hello world" });
        let result = def.handler.call(input).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "hello world");
    }

    #[tokio::test]
    async fn test_tool_handler_error_propagation() {
        struct FailHandler;
        impl ToolHandler for FailHandler {
            fn call(
                &self,
                _input: Value,
            ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
                Box::pin(async { Err(McpSdkError::HandlerError("deliberate failure".to_string())) })
            }
        }

        let def = ToolDefinition::new(
            "fail_tool",
            "Always fails",
            serde_json::json!({ "type": "object" }),
            FailHandler,
        );

        let result = def.handler.call(serde_json::json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "handler error: deliberate failure");
    }

    #[test]
    fn test_tool_definition_input_schema() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "User name" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name"]
        });

        let def = ToolDefinition::new("profile", "User profile", schema.clone(), AddHandler);

        assert_eq!(def.input_schema, schema);
        assert_eq!(def.input_schema["type"], "object");
        assert_eq!(def.input_schema["required"][0], "name");
        assert_eq!(def.input_schema["properties"]["age"]["minimum"], 0);
    }
}
