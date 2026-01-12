# Tracking Costs and Usage

Monitoring token usage and financial cost is critical for production AI systems. The Claude Agent Rust SDK provides detailed metrics for every conversation turn.

## Usage Data

At the end of each turn, the SDK yields a `ResultMessage` containing usage statistics.

```rust
use claude_agent_types::Message;

// ... in your message stream loop
if let Message::Result(res) = message {
    println!("Total Cost: ${:.4}", res.total_cost_usd.unwrap_or(0.0));
    println!("Num Turns: {}", res.num_turns);
    
    if let Some(usage) = res.usage {
        println!("Input tokens: {:?}", usage.get("input_tokens"));
        println!("Output tokens: {:?}", usage.get("output_tokens"));
    }
}
```

## Budgeting

You can set a maximum budget (in USD) for a session in `ClaudeAgentOptions`. Once reached, the agent will stop executing.

```rust
let options = ClaudeAgentOptions {
    max_budget_usd: Some(5.00), // Stop after $5.00
    ..Default::default()
};
```

## Max Turns

As an additional safety measure, you can limit the number of turns (API calls) per query.

```rust
let options = ClaudeAgentOptions {
    max_turns: Some(10),
    ..Default::default()
};
```

## Implementation Notes

- Costs are calculated by the CLI based on current Anthropic model pricing.
- Token usage counts include both user prompts and assistant completions.
