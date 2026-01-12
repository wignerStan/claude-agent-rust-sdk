# Structured Outputs in the SDK

The Claude Agent can be forced to return results in a specific JSON format using JSON Schema. This is ideal for extracting data from text or building automated pipelines.

## Configuring Structured Output

Use the `output_format` field in `ClaudeAgentOptions`.

```rust
use serde_json::json;

let options = ClaudeAgentOptions {
    output_format: Some(json!({
        "type": "json_schema",
        "json_schema": {
            "name": "analysis_result",
            "schema": {
                "type": "object",
                "properties": {
                    "severity": { "type": "string", "enum": ["low", "medium", "high"] },
                    "summary": { "type": "string" }
                }
            }
        }
    })),
    ..Default::default()
};
```

## Retrieving the Result

The structured result is available in the `ResultMessage` at the end of the conversation.

```rust
if let Message::Result(res) = message {
    if let Some(json) = res.structured_output {
        println!("Structured result: {}", json);
    }
}
```

## When to use Structured Output

- **Data Extraction**: Converting natural language logs into structured records.
- **Classification**: Categorizing issues or sentiment.
- **Form Filling**: Guiding the agent to collect specific pieces of information.

> [!IMPORTANT]
> When `output_format` is set, the agent may be less conversational as it focuses on generating valid JSON.
