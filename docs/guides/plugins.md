# Plugins in the SDK

Plugins allow you to extend the Claude CLI with custom logic, slash commands, and tools that are reusable across different projects.

## Configuring Plugins

Plugins are registered in `ClaudeAgentOptions`. Currently, the SDK supports local plugins.

```rust
use serde_json::json;

let options = ClaudeAgentOptions {
    plugins: vec![
        json!({
            "type": "local",
            "path": "./my-plugin"
        })
    ],
    ..Default::default()
};
```

## What Plugins Can Do

1. **Add Tools**: New MCP tools can be bundled in a plugin.
2. **Add Slash Commands**: Define new `/commands`.
3. **Customize Environment**: Set environment variables or aliases.

## SDK vs. CLI Plugins

While the SDK allows you to define tools directly in code (via `SdkMcpServer`), plugins are better for functionality that you want to share between your SDK application and interactive use of the `claude` CLI.

## Plugin Discovery

The CLI will attempt to load plugins from the specified paths. Ensure that the paths are reachable from the environment where the CLI process is running.
