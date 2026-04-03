use claude_agent::types::config::*;
use claude_agent::types::ClaudeAgentOptions;
use std::collections::HashMap;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// PermissionMode
// ---------------------------------------------------------------------------

#[test]
fn permission_mode_all_variants_serde_roundtrip() {
    let variants = vec![
        PermissionMode::Default,
        PermissionMode::AcceptEdits,
        PermissionMode::Plan,
        PermissionMode::BypassPermissions,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: PermissionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

#[test]
fn permission_mode_serde_values() {
    assert_eq!(serde_json::to_string(&PermissionMode::Default).unwrap(), r#""default""#);
    assert_eq!(serde_json::to_string(&PermissionMode::AcceptEdits).unwrap(), r#""acceptEdits""#);
    assert_eq!(serde_json::to_string(&PermissionMode::Plan).unwrap(), r#""plan""#);
    assert_eq!(
        serde_json::to_string(&PermissionMode::BypassPermissions).unwrap(),
        r#""bypassPermissions""#
    );
}

#[test]
fn permission_mode_display() {
    assert_eq!(format!("{}", PermissionMode::Default), "default");
    assert_eq!(format!("{}", PermissionMode::AcceptEdits), "acceptEdits");
    assert_eq!(format!("{}", PermissionMode::Plan), "plan");
    assert_eq!(format!("{}", PermissionMode::BypassPermissions), "bypassPermissions");
}

#[test]
fn permission_mode_equality() {
    assert_eq!(PermissionMode::Default, PermissionMode::Default);
    assert_ne!(PermissionMode::Default, PermissionMode::Plan);
}

// ---------------------------------------------------------------------------
// McpTransportType
// ---------------------------------------------------------------------------

#[test]
fn mcp_transport_type_all_variants_serde_roundtrip() {
    let variants = vec![
        McpTransportType::Stdio,
        McpTransportType::Http,
        McpTransportType::Sse,
        McpTransportType::Auto,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: McpTransportType = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

#[test]
fn mcp_transport_type_serde_values() {
    assert_eq!(serde_json::to_string(&McpTransportType::Stdio).unwrap(), r#""stdio""#);
    assert_eq!(serde_json::to_string(&McpTransportType::Http).unwrap(), r#""http""#);
    assert_eq!(serde_json::to_string(&McpTransportType::Sse).unwrap(), r#""sse""#);
    assert_eq!(serde_json::to_string(&McpTransportType::Auto).unwrap(), r#""auto""#);
}

#[test]
fn mcp_transport_type_default() {
    assert_eq!(McpTransportType::default(), McpTransportType::Stdio);
}

#[test]
fn mcp_transport_type_display() {
    assert_eq!(format!("{}", McpTransportType::Stdio), "stdio");
    assert_eq!(format!("{}", McpTransportType::Http), "http");
    assert_eq!(format!("{}", McpTransportType::Sse), "sse");
    assert_eq!(format!("{}", McpTransportType::Auto), "auto");
}

#[test]
fn mcp_transport_type_equality() {
    assert_eq!(McpTransportType::Stdio, McpTransportType::Stdio);
    assert_ne!(McpTransportType::Stdio, McpTransportType::Http);
}

// ---------------------------------------------------------------------------
// SettingSource
// ---------------------------------------------------------------------------

#[test]
fn setting_source_all_variants_serde_roundtrip() {
    let variants = vec![SettingSource::User, SettingSource::Project, SettingSource::Local];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let _back: SettingSource = serde_json::from_str(&json).unwrap();
    }
}

// ---------------------------------------------------------------------------
// McpServerConfig
// ---------------------------------------------------------------------------

#[test]
fn mcp_server_config_default() {
    let cfg = McpServerConfig::default();
    assert_eq!(cfg.transport, McpTransportType::Stdio);
    assert!(cfg.command.is_none());
    assert!(cfg.args.is_empty());
    assert!(cfg.url.is_none());
    assert!(cfg.timeout_secs.is_none());
    assert!(cfg.env.is_empty());
}

#[test]
fn mcp_server_config_full_serde_roundtrip() {
    let mut env = HashMap::new();
    env.insert("KEY".to_string(), "VALUE".to_string());
    let cfg = McpServerConfig {
        transport: McpTransportType::Stdio,
        command: Some("npx".to_string()),
        args: vec!["-y".to_string(), "@anthropic/mcp".to_string()],
        url: None,
        timeout_secs: Some(30),
        env,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: McpServerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.command, Some("npx".to_string()));
    assert_eq!(back.args, vec!["-y", "@anthropic/mcp"]);
    assert_eq!(back.timeout_secs, Some(30));
    assert_eq!(back.env.get("KEY").unwrap(), "VALUE");
}

#[test]
fn mcp_server_config_stdio_with_args() {
    let cfg = McpServerConfig {
        transport: McpTransportType::Stdio,
        command: Some("node".to_string()),
        args: vec!["server.js".to_string()],
        url: None,
        timeout_secs: None,
        env: HashMap::new(),
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: McpServerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.args.len(), 1);
    assert_eq!(back.args[0], "server.js");
}

// ---------------------------------------------------------------------------
// AgentDefinition
// ---------------------------------------------------------------------------

#[test]
fn agent_definition_full_serde_roundtrip() {
    let def = AgentDefinition {
        description: "A test agent".to_string(),
        prompt: "You are a test agent.".to_string(),
        tools: Some(vec!["Read".to_string(), "Write".to_string()]),
        model: Some("claude-sonnet-4-20250514".to_string()),
        disallowed_tools: None,
        skills: None,
        memory: None,
        mcp_servers: None,
        initial_prompt: None,
        max_turns: None,
    };
    let json = serde_json::to_string(&def).unwrap();
    let back: AgentDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(back.description, "A test agent");
    assert_eq!(back.prompt, "You are a test agent.");
    assert_eq!(back.tools.as_ref().unwrap().len(), 2);
    assert_eq!(back.model, Some("claude-sonnet-4-20250514".to_string()));
}

#[test]
fn agent_definition_minimal_serde_roundtrip() {
    let def = AgentDefinition {
        description: "Minimal agent".to_string(),
        prompt: "Be minimal.".to_string(),
        tools: None,
        model: None,
        disallowed_tools: None,
        skills: None,
        memory: None,
        mcp_servers: None,
        initial_prompt: None,
        max_turns: None,
    };
    let json = serde_json::to_string(&def).unwrap();
    let back: AgentDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(back.description, "Minimal agent");
    assert!(back.tools.is_none());
    assert!(back.model.is_none());
}

// ---------------------------------------------------------------------------
// ClaudeAgentOptions
// ---------------------------------------------------------------------------

#[test]
fn claude_agent_options_default() {
    let opts = ClaudeAgentOptions::default();
    assert!(opts.tools.is_none());
    assert!(opts.allowed_tools.is_empty());
    assert!(opts.system_prompt.is_none());
    assert!(opts.mcp_servers.is_empty());
    assert!(opts.permission_mode.is_none());
    assert!(!opts.continue_conversation);
    assert!(opts.resume.is_none());
    assert!(opts.max_turns.is_none());
    assert!(opts.max_budget_usd.is_none());
    assert!(opts.disallowed_tools.is_empty());
    assert!(opts.model.is_none());
    assert!(opts.fallback_model.is_none());
    assert!(opts.betas.is_empty());
    assert!(opts.cwd.is_none());
    assert!(opts.cli_path.is_none());
    assert!(opts.add_dirs.is_empty());
    assert!(opts.env.is_empty());
    assert!(opts.extra_args.is_empty());
    assert!(opts.max_thinking_tokens.is_none());
    assert!(!opts.include_partial_messages);
    assert!(!opts.fork_session);
    assert!(opts.agents.is_none());
    assert!(opts.sandbox.is_none());
    assert!(opts.plugins.is_empty());
}

#[test]
fn claude_agent_options_full_serde_roundtrip() {
    let mut mcp_servers = HashMap::new();
    mcp_servers.insert("server1".to_string(), serde_json::json!({"command": "npx"}));
    let mut env = HashMap::new();
    env.insert("KEY".to_string(), "VALUE".to_string());
    let mut extra_args = HashMap::new();
    extra_args.insert("key1".to_string(), Some("val1".to_string()));
    extra_args.insert("key2".to_string(), None);
    let mut agents = HashMap::new();
    agents.insert(
        "reviewer".to_string(),
        AgentDefinition {
            description: "Code reviewer".to_string(),
            prompt: "Review code.".to_string(),
            tools: Some(vec!["Read".to_string()]),
            model: None,
            disallowed_tools: None,
            skills: None,
            memory: None,
            mcp_servers: None,
            initial_prompt: None,
            max_turns: None,
        },
    );

    let opts = ClaudeAgentOptions {
        tools: Some(ToolsConfig::List(vec!["Read".to_string(), "Write".to_string()])),
        allowed_tools: vec!["Bash".to_string()],
        system_prompt: Some(SystemPromptConfig::Text("You are helpful.".to_string())),
        mcp_servers,
        permission_mode: Some(PermissionMode::AcceptEdits),
        continue_conversation: true,
        resume: Some("session-123".to_string()),
        max_turns: Some(50),
        max_budget_usd: Some(5.0),
        disallowed_tools: vec!["DangerousTool".to_string()],
        model: Some("claude-sonnet-4-20250514".to_string()),
        fallback_model: Some("claude-haiku-4-20250414".to_string()),
        betas: vec!["beta1".to_string()],
        permission_prompt_tool_name: Some("ToolPrompt".to_string()),
        cwd: Some(PathBuf::from("/workspace")),
        cli_path: Some(PathBuf::from("/usr/local/bin/claude")),
        settings: Some("settings.json".to_string()),
        add_dirs: vec![PathBuf::from("/extra")],
        env,
        extra_args,
        max_buffer_size: Some(1024),
        include_partial_messages: true,
        fork_session: true,
        agents: Some(agents),
        setting_sources: Some(vec![SettingSource::User, SettingSource::Project]),
        sandbox: Some(SandboxSettings {
            enabled: true,
            auto_allow_bash_if_sandboxed: true,
            excluded_commands: vec!["rm".to_string()],
            allow_unsandboxed_commands: false,
            network: None,
            ignore_violations: None,
            enable_weaker_nested_sandbox: false,
        }),
        plugins: vec![PluginConfig::Local { path: PathBuf::from("/plugins/test") }],
        max_thinking_tokens: Some(8000),
        output_format: Some(serde_json::json!({"type": "stream_json"})),
        enable_file_checkpointing: true,
        effort: None,
        thinking: None,
        task_budget: None,
        session_id: None,
        strict_mcp_config: false,
    };

    let json = serde_json::to_string(&opts).unwrap();
    let back: ClaudeAgentOptions = serde_json::from_str(&json).unwrap();

    assert_eq!(back.permission_mode, Some(PermissionMode::AcceptEdits));
    assert!(back.continue_conversation);
    assert_eq!(back.resume, Some("session-123".to_string()));
    assert_eq!(back.max_turns, Some(50));
    assert_eq!(back.max_budget_usd, Some(5.0));
    assert_eq!(back.model, Some("claude-sonnet-4-20250514".to_string()));
    assert_eq!(back.fallback_model, Some("claude-haiku-4-20250414".to_string()));
    assert_eq!(back.betas.len(), 1);
    assert_eq!(back.cwd, Some(PathBuf::from("/workspace")));
    assert_eq!(back.max_buffer_size, Some(1024));
    assert!(back.include_partial_messages);
    assert!(back.fork_session);
    assert!(back.agents.is_some());
    assert!(back.sandbox.is_some());
    assert_eq!(back.plugins.len(), 1);
    assert_eq!(back.max_thinking_tokens, Some(8000));
}

#[test]
fn claude_agent_options_skip_none_serialization() {
    let opts = ClaudeAgentOptions::default();
    let json = serde_json::to_string(&opts).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = parsed.as_object().unwrap();
    assert!(!obj.contains_key("tools"));
    assert!(!obj.contains_key("systemPrompt"));
    assert!(!obj.contains_key("permissionMode"));
    assert!(!obj.contains_key("resume"));
    assert!(!obj.contains_key("maxTurns"));
    assert!(!obj.contains_key("model"));
}

#[test]
fn claude_agent_options_with_env() {
    let mut env = HashMap::new();
    env.insert("API_KEY".to_string(), "secret".to_string());
    let opts = ClaudeAgentOptions { env, ..Default::default() };
    let json = serde_json::to_string(&opts).unwrap();
    let back: ClaudeAgentOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.env.get("API_KEY").unwrap(), "secret");
}

#[test]
fn claude_agent_options_with_mcp_servers() {
    let mut mcp = HashMap::new();
    mcp.insert("s1".to_string(), serde_json::json!({"command": "npx"}));
    let opts = ClaudeAgentOptions { mcp_servers: mcp, ..Default::default() };
    let json = serde_json::to_string(&opts).unwrap();
    let back: ClaudeAgentOptions = serde_json::from_str(&json).unwrap();
    assert!(back.mcp_servers.contains_key("s1"));
}

// ---------------------------------------------------------------------------
// ToolsConfig
// ---------------------------------------------------------------------------

#[test]
fn tools_config_list_serde_roundtrip() {
    let cfg = ToolsConfig::List(vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()]);
    let json = serde_json::to_string(&cfg).unwrap();
    let back: ToolsConfig = serde_json::from_str(&json).unwrap();
    match back {
        ToolsConfig::List(tools) => assert_eq!(tools.len(), 3),
        ToolsConfig::Preset(_) => panic!("expected List variant"),
    }
}

#[test]
fn tools_config_preset_serde_roundtrip() {
    let json = r#"{"type":"preset","preset":"claude_code"}"#;
    let back: ToolsConfig = serde_json::from_str(json).unwrap();
    match back {
        ToolsConfig::Preset(_) => {},
        ToolsConfig::List(_) => panic!("expected Preset variant"),
    }
}

#[test]
fn tools_config_list_empty() {
    let cfg = ToolsConfig::List(vec![]);
    let json = serde_json::to_string(&cfg).unwrap();
    assert_eq!(json, "[]");
}

// ---------------------------------------------------------------------------
// SystemPromptConfig
// ---------------------------------------------------------------------------

#[test]
fn system_prompt_config_text_serde_roundtrip() {
    let cfg = SystemPromptConfig::Text("You are a helpful assistant.".to_string());
    let json = serde_json::to_string(&cfg).unwrap();
    let back: SystemPromptConfig = serde_json::from_str(&json).unwrap();
    match back {
        SystemPromptConfig::Text(s) => assert_eq!(s, "You are a helpful assistant."),
        SystemPromptConfig::Preset(_) => panic!("expected Text variant"),
    }
}

#[test]
fn system_prompt_config_preset_serde_roundtrip() {
    let json = r#"{"type":"preset","preset":"claude_code","append":"Additional instructions."}"#;
    let back: SystemPromptConfig = serde_json::from_str(json).unwrap();
    match back {
        SystemPromptConfig::Preset(_) => {},
        SystemPromptConfig::Text(_) => panic!("expected Preset variant"),
    }
}

// ---------------------------------------------------------------------------
// PluginConfig
// ---------------------------------------------------------------------------

#[test]
fn plugin_config_local_serde_roundtrip() {
    let cfg = PluginConfig::Local { path: PathBuf::from("/plugins/test") };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: PluginConfig = serde_json::from_str(&json).unwrap();
    match back {
        PluginConfig::Local { path } => assert_eq!(path, PathBuf::from("/plugins/test")),
    }
}

// ---------------------------------------------------------------------------
// SandboxSettings
// ---------------------------------------------------------------------------

#[test]
fn sandbox_settings_default() {
    let settings = SandboxSettings::default();
    assert!(!settings.enabled);
    assert!(!settings.auto_allow_bash_if_sandboxed);
    assert!(settings.excluded_commands.is_empty());
    assert!(!settings.allow_unsandboxed_commands);
    assert!(settings.network.is_none());
    assert!(settings.ignore_violations.is_none());
    assert!(!settings.enable_weaker_nested_sandbox);
}

#[test]
fn sandbox_settings_full_serde_roundtrip() {
    let settings = SandboxSettings {
        enabled: true,
        auto_allow_bash_if_sandboxed: true,
        excluded_commands: vec!["rm".to_string(), "sudo".to_string()],
        allow_unsandboxed_commands: false,
        network: Some(SandboxNetworkConfig {
            allow_unix_sockets: vec!["/tmp/sandbox".to_string()],
            allow_all_unix_sockets: false,
            allow_local_binding: true,
            http_proxy_port: Some(8080),
            socks_proxy_port: None,
        }),
        ignore_violations: Some(SandboxIgnoreViolations {
            file: vec!["/etc/hosts".to_string()],
            network: vec![],
        }),
        enable_weaker_nested_sandbox: true,
    };
    let json = serde_json::to_string(&settings).unwrap();
    let back: SandboxSettings = serde_json::from_str(&json).unwrap();
    assert!(back.enabled);
    assert!(back.auto_allow_bash_if_sandboxed);
    assert_eq!(back.excluded_commands.len(), 2);
    assert!(!back.allow_unsandboxed_commands);
    assert!(back.network.is_some());
    assert!(back.ignore_violations.is_some());
    assert!(back.enable_weaker_nested_sandbox);
}

// ---------------------------------------------------------------------------
// SandboxNetworkConfig
// ---------------------------------------------------------------------------

#[test]
fn sandbox_network_config_default() {
    let cfg = SandboxNetworkConfig::default();
    assert!(cfg.allow_unix_sockets.is_empty());
    assert!(!cfg.allow_all_unix_sockets);
    assert!(!cfg.allow_local_binding);
    assert!(cfg.http_proxy_port.is_none());
    assert!(cfg.socks_proxy_port.is_none());
}

#[test]
fn sandbox_network_config_serde() {
    let cfg = SandboxNetworkConfig {
        allow_unix_sockets: vec!["/tmp/s1".to_string()],
        allow_all_unix_sockets: true,
        allow_local_binding: true,
        http_proxy_port: Some(3128),
        socks_proxy_port: Some(1080),
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: SandboxNetworkConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.allow_unix_sockets.len(), 1);
    assert!(back.allow_all_unix_sockets);
    assert!(back.allow_local_binding);
    assert_eq!(back.http_proxy_port, Some(3128));
    assert_eq!(back.socks_proxy_port, Some(1080));
}

// ---------------------------------------------------------------------------
// SandboxIgnoreViolations
// ---------------------------------------------------------------------------

#[test]
fn sandbox_ignore_violations_default() {
    let v = SandboxIgnoreViolations::default();
    assert!(v.file.is_empty());
    assert!(v.network.is_empty());
}

#[test]
fn sandbox_ignore_violations_serde() {
    let v = SandboxIgnoreViolations {
        file: vec!["/etc/passwd".to_string()],
        network: vec!["10.0.0.0/8".to_string()],
    };
    let json = serde_json::to_string(&v).unwrap();
    let back: SandboxIgnoreViolations = serde_json::from_str(&json).unwrap();
    assert_eq!(back.file.len(), 1);
    assert_eq!(back.network.len(), 1);
}

// ---------------------------------------------------------------------------
// JsonSchema smoke tests
// ---------------------------------------------------------------------------

#[test]
fn json_schema_for_permission_mode() {
    let schema = schemars::schema_for!(PermissionMode);
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("PermissionMode"));
}

#[test]
fn json_schema_for_mcp_server_config() {
    let schema = schemars::schema_for!(McpServerConfig);
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("McpServerConfig"));
}

#[test]
fn json_schema_for_agent_definition() {
    let schema = schemars::schema_for!(AgentDefinition);
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("AgentDefinition"));
}

#[test]
fn json_schema_for_sandbox_settings() {
    let schema = schemars::schema_for!(SandboxSettings);
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("SandboxSettings"));
}
