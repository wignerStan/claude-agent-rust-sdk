//! Property-based tests for MCP types using proptest.

use proptest::prelude::*;
use serde_json::json;

use claude_agent_mcp::manager::ToolInfo;

proptest! {
    /// Test that ToolInfo serialization round-trips correctly.
    #[test]
    fn test_tool_info_roundtrip(
        name in "[a-z_][a-z0-9_]{0,30}",
        description in prop::option::of("[a-zA-Z0-9 ,.!?]{0,100}")
    ) {
        let tool = ToolInfo {
            name: name.clone(),
            description: description.clone(),
            input_schema: json!({"type": "object"}),
        };

        let json_str = serde_json::to_string(&tool).unwrap();
        let deserialized: ToolInfo = serde_json::from_str(&json_str).unwrap();

        prop_assert_eq!(deserialized.name, name);
        prop_assert_eq!(deserialized.description, description);
    }

    /// Test that JSON serialization never panics for valid inputs.
    #[test]
    fn test_json_serialization_no_panic(
        key in "[a-z]{1,20}",
        value in "[a-zA-Z0-9]{0,50}"
    ) {
        let obj = json!({
            key: value
        });

        let _ = obj.to_string();
        prop_assert!(true);
    }

    /// Test that tool names have valid format.
    #[test]
    fn test_tool_name_format(name in "[a-z_][a-z0-9_]{0,30}") {
        // Tool names should start with a letter or underscore
        prop_assert!(name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_'));

        // Tool names should only contain alphanumeric and underscore
        prop_assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    }
}

#[cfg(test)]
mod rate_limiter_tests {
    use claude_agent_mcp::rate_limiter::{RateLimitConfig, RateLimiter};
    use proptest::prelude::*;

    proptest! {
        /// Test that rate limiter creation doesn't panic for valid inputs.
        #[test]
        fn test_rate_limiter_creation(
            rps in 1u32..1000,
            burst in 1u32..1000
        ) {
            let config = RateLimitConfig::new(rps, burst);
            let limiter = RateLimiter::new(config);

            // Should always be able to create a valid rate limiter
            prop_assert_eq!(limiter.config().requests_per_second, rps);
            prop_assert_eq!(limiter.config().burst_size, burst);
        }

        /// Test that initial burst always succeeds.
        #[test]
        fn test_initial_burst_succeeds(burst in 1u32..100) {
            let config = RateLimitConfig::new(100, burst);
            let limiter = RateLimiter::new(config);

            // First `burst` requests should succeed
            for _ in 0..burst {
                prop_assert!(limiter.check());
            }
        }
    }
}

#[cfg(test)]
mod transport_config_tests {
    use claude_agent_types::config::{McpServerConfig, McpTransportType};
    use proptest::prelude::*;

    proptest! {
        /// Test that transport configuration can be created with any valid URL.
        #[test]
        fn test_http_config_creation(
            url in "http://[a-z]{1,10}\\.[a-z]{2,3}(:[0-9]{1,5})?"
        ) {
            let config = McpServerConfig {
                transport: McpTransportType::Http,
                url: Some(url.clone()),
                ..Default::default()
            };

            prop_assert_eq!(config.url, Some(url));
            prop_assert_eq!(config.transport, McpTransportType::Http);
        }

        /// Test that timeout configuration is preserved.
        #[test]
        fn test_timeout_config(timeout in 1u64..3600) {
            let config = McpServerConfig {
                transport: McpTransportType::Http,
                url: Some("http://localhost".to_string()),
                timeout_secs: Some(timeout),
                ..Default::default()
            };

            prop_assert_eq!(config.timeout_secs, Some(timeout));
        }
    }
}
