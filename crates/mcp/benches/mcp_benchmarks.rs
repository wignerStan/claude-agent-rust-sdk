//! MCP performance benchmarks.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

fn benchmark_json_parsing(c: &mut Criterion) {
    let tool_response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "tools": [
                {
                    "name": "test_tool",
                    "description": "A test tool for benchmarking",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "count": { "type": "integer" }
                        },
                        "required": ["name"]
                    }
                }
            ]
        }
    });

    let json_str = tool_response.to_string();

    c.bench_function("json_parse_tool_response", |b| {
        b.iter(|| {
            let _: serde_json::Value = serde_json::from_str(black_box(&json_str)).unwrap();
        })
    });

    c.bench_function("json_serialize_tool_response", |b| {
        b.iter(|| {
            let _ = black_box(&tool_response).to_string();
        })
    });
}

fn benchmark_rate_limiter(c: &mut Criterion) {
    use claude_agent_mcp::rate_limiter::{RateLimitConfig, RateLimiter};

    let limiter = RateLimiter::new(RateLimitConfig::permissive());

    c.bench_function("rate_limiter_check", |b| {
        b.iter(|| {
            let _ = black_box(&limiter).check();
        })
    });
}

fn benchmark_tool_info_serialization(c: &mut Criterion) {
    use claude_agent_mcp::ToolInfo;

    let tool_info = ToolInfo {
        name: "test_tool".to_string(),
        description: Some(
            "A test tool for benchmarking JSON serialization performance".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" },
                "options": {
                    "type": "object",
                    "properties": {
                        "verbose": { "type": "boolean" },
                        "limit": { "type": "integer" }
                    }
                }
            },
            "required": ["input"]
        }),
    };

    c.bench_function("tool_info_serialize", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(black_box(&tool_info)).unwrap();
        })
    });

    let json_str = serde_json::to_string(&tool_info).unwrap();
    c.bench_function("tool_info_deserialize", |b| {
        b.iter(|| {
            let _: ToolInfo = serde_json::from_str(black_box(&json_str)).unwrap();
        })
    });
}

fn benchmark_transport_factory(c: &mut Criterion) {
    use claude_agent_mcp::create_mcp_server;
    use claude_agent_types::config::{McpServerConfig, McpTransportType};

    c.bench_function("create_http_server", |b| {
        b.iter(|| {
            let config = McpServerConfig {
                transport: McpTransportType::Http,
                url: Some("http://localhost:8080".to_string()),
                ..Default::default()
            };
            let _ = create_mcp_server(black_box("test".to_string()), black_box(config));
        })
    });
}

criterion_group!(
    benches,
    benchmark_json_parsing,
    benchmark_rate_limiter,
    benchmark_tool_info_serialization,
    benchmark_transport_factory
);
criterion_main!(benches);
