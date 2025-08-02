//! Comprehensive tests for client configuration management

use mcp_helper::client::{
    detect_clients, ClaudeCodeClient, ClaudeDesktopClient, ClientRegistry, CursorClient,
    McpClient, ServerConfig, VSCodeClient, WindsurfClient,
};
use std::collections::HashMap;

/// Helper to create test server configurations
#[allow(dead_code)]
fn create_test_server_config(name: &str) -> ServerConfig {
    ServerConfig {
        command: format!("{name}-cmd"),
        args: vec![format!("{name}-arg1"), format!("{name}-arg2")],
        env: {
            let mut env = HashMap::new();
            env.insert(format!("{}_KEY", name.to_uppercase()), format!("{name}-value"));
            env
        },
    }
}

#[test]
fn test_all_client_types_have_unique_names() {
    let clients = detect_clients();
    let mut names = Vec::new();

    for client in &clients {
        let name = client.name();
        assert!(!names.contains(&name), "Duplicate client name found: {name}");
        names.push(name);
    }

    // Verify expected client names
    assert!(names.contains(&"Claude Code"));
    assert!(names.contains(&"Claude Desktop"));
    assert!(names.contains(&"Cursor"));
    assert!(names.contains(&"VS Code"));
    assert!(names.contains(&"Windsurf"));
}

#[test]
fn test_all_client_types_have_valid_config_paths() {
    let clients = detect_clients();

    for client in &clients {
        let path = client.config_path();
        
        // Path should not be empty
        let name = client.name();
        assert!(!path.as_os_str().is_empty(), "{name} has empty config path");
        
        // Path should contain appropriate file extension
        let path_str = path.to_string_lossy();
        assert!(
            path_str.ends_with(".json") || 
            path_str.ends_with(".yaml") || 
            path_str.ends_with(".yml") ||
            path_str.ends_with(".toml") ||
            path_str.contains("config"), // Some clients might use config directories
            "{} has unexpected config path format: {}", 
            client.name(), 
            path_str
        );
    }
}

#[test]
fn test_client_specific_paths() {
    // Test Claude Desktop
    let claude = ClaudeDesktopClient::new();
    let claude_path = claude.config_path();
    assert!(claude_path.to_string_lossy().contains("claude_desktop_config.json"));

    // Test Claude Code
    let claude_code = ClaudeCodeClient::new();
    let claude_code_path = claude_code.config_path();
    assert!(claude_code_path.to_string_lossy().contains(".claude.json"));

    // Test Cursor
    let cursor = CursorClient::new();
    let cursor_path = cursor.config_path();
    assert!(cursor_path.to_string_lossy().contains("Cursor") || 
            cursor_path.to_string_lossy().contains("cursor"));

    // Test VS Code
    let vscode = VSCodeClient::new();
    let vscode_path = vscode.config_path();
    assert!(vscode_path.to_string_lossy().contains("Code") || 
            vscode_path.to_string_lossy().contains("vscode"));

    // Test Windsurf
    let windsurf = WindsurfClient::new();
    let windsurf_path = windsurf.config_path();
    assert!(windsurf_path.to_string_lossy().contains("windsurf"));
}

#[test]
fn test_server_config_equality_and_cloning() {
    let config1 = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string(), "--port".to_string(), "3000".to_string()],
        env: {
            let mut env = HashMap::new();
            env.insert("NODE_ENV".to_string(), "production".to_string());
            env.insert("API_KEY".to_string(), "secret123".to_string());
            env
        },
    };

    // Test clone
    let config2 = config1.clone();
    assert_eq!(config1, config2);

    // Test inequality with different command
    let mut config3 = config1.clone();
    config3.command = "python".to_string();
    assert_ne!(config1, config3);

    // Test inequality with different args
    let mut config4 = config1.clone();
    config4.args.push("extra-arg".to_string());
    assert_ne!(config1, config4);

    // Test inequality with different env
    let mut config5 = config1.clone();
    config5.env.insert("EXTRA_VAR".to_string(), "value".to_string());
    assert_ne!(config1, config5);
}

#[test]
fn test_server_config_json_serialization() {
    let original = ServerConfig {
        command: "docker".to_string(),
        args: vec![
            "run".to_string(),
            "-it".to_string(),
            "--rm".to_string(),
            "-v".to_string(),
            "/host/path:/container/path".to_string(),
            "image:tag".to_string(),
        ],
        env: {
            let mut env = HashMap::new();
            env.insert("DOCKER_HOST".to_string(), "unix:///var/run/docker.sock".to_string());
            env.insert("COMPOSE_PROJECT_NAME".to_string(), "mcp-test".to_string());
            env
        },
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&original).unwrap();
    
    // Verify JSON structure
    assert!(json.contains("\"command\": \"docker\""));
    assert!(json.contains("\"run\""));
    assert!(json.contains("\"DOCKER_HOST\""));
    
    // Deserialize back
    let deserialized: ServerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_server_config_edge_cases() {
    // Empty command (should be invalid in real use)
    let config = ServerConfig {
        command: String::new(),
        args: vec![],
        env: HashMap::new(),
    };
    assert!(config.command.is_empty());

    // Command with spaces
    let config = ServerConfig {
        command: "C:\\Program Files\\Node\\node.exe".to_string(),
        args: vec![],
        env: HashMap::new(),
    };
    assert!(config.command.contains(' '));

    // Args with special characters
    let config = ServerConfig {
        command: "bash".to_string(),
        args: vec![
            "-c".to_string(),
            "echo 'Hello, World!' && exit 0".to_string(),
        ],
        env: HashMap::new(),
    };
    assert!(config.args[1].contains('\''));
    assert!(config.args[1].contains('&'));

    // Environment variables with special values
    let mut env = HashMap::new();
    env.insert("EMPTY".to_string(), String::new());
    env.insert("SPACES".to_string(), "   ".to_string());
    env.insert("NEWLINE".to_string(), "line1\nline2".to_string());
    env.insert("UNICODE".to_string(), "Hello 世界 🌍".to_string());
    
    let config = ServerConfig {
        command: "test".to_string(),
        args: vec![],
        env,
    };
    
    assert_eq!(config.env["EMPTY"], "");
    assert_eq!(config.env["SPACES"], "   ");
    assert!(config.env["NEWLINE"].contains('\n'));
    assert!(config.env["UNICODE"].contains('世'));
}

#[test]
fn test_client_registry_with_all_real_clients() {
    let mut registry = ClientRegistry::new();
    
    // Register all real client types
    registry.register(Box::new(ClaudeCodeClient::new()));
    registry.register(Box::new(ClaudeDesktopClient::new()));
    registry.register(Box::new(CursorClient::new()));
    registry.register(Box::new(VSCodeClient::new()));
    registry.register(Box::new(WindsurfClient::new()));

    assert_eq!(registry.clients.len(), 5);

    // Test name lookups
    assert!(registry.get_by_name("Claude Code").is_some());
    assert!(registry.get_by_name("claude desktop").is_some()); // Case insensitive
    assert!(registry.get_by_name("CURSOR").is_some());
    assert!(registry.get_by_name("vs code").is_some());
    assert!(registry.get_by_name("Windsurf").is_some());
    assert!(registry.get_by_name("Unknown Client").is_none());
}

#[test]
fn test_detect_clients_consistency() {
    // Run detect_clients multiple times and ensure consistency
    let clients1 = detect_clients();
    let clients2 = detect_clients();
    let clients3 = detect_clients();

    assert_eq!(clients1.len(), clients2.len());
    assert_eq!(clients2.len(), clients3.len());

    // Verify same client names in same order
    for i in 0..clients1.len() {
        assert_eq!(clients1[i].name(), clients2[i].name());
        assert_eq!(clients2[i].name(), clients3[i].name());
    }
}

#[test]
fn test_platform_specific_config_paths() {
    let clients = detect_clients();

    for client in &clients {
        let path = client.config_path();
        let path_str = path.to_string_lossy();

        #[cfg(target_os = "windows")]
        {
            // Windows paths should use appropriate directories
            assert!(
                path_str.contains("AppData") ||
                path_str.contains("ProgramData") ||
                path_str.contains("Documents") ||
                path_str.contains("\\") || // At least use Windows path separator
                path.is_relative(), // Or be a relative path
                "{} doesn't use Windows-appropriate path: {}",
                client.name(),
                path_str
            );
        }

        #[cfg(target_os = "macos")]
        {
            // macOS paths should use appropriate directories
            assert!(
                path_str.contains("Library") ||
                path_str.contains("Application Support") ||
                path_str.contains(".config") ||
                path_str.starts_with("/Users") ||
                path_str.starts_with("~") ||
                path.is_relative(),
                "{} doesn't use macOS-appropriate path: {}",
                client.name(),
                path_str
            );
        }

        #[cfg(target_os = "linux")]
        {
            // Linux paths should use appropriate directories
            assert!(
                path_str.contains(".config") ||
                path_str.contains(".local") ||
                path_str.starts_with("/home") ||
                path_str.starts_with("~") ||
                path.is_relative(),
                "{} doesn't use Linux-appropriate path: {}",
                client.name(),
                path_str
            );
        }
    }
}

#[test]
fn test_server_config_with_different_command_types() {
    // NPM/NPX server
    let npm_config = ServerConfig {
        command: "npx".to_string(),
        args: vec![
            "-y".to_string(),
            "@modelcontextprotocol/server-filesystem".to_string(),
            "/path/to/files".to_string(),
        ],
        env: HashMap::new(),
    };

    // Python server
    let python_config = ServerConfig {
        command: "python3".to_string(),
        args: vec![
            "-m".to_string(),
            "mcp_server.main".to_string(),
            "--config".to_string(),
            "server.yaml".to_string(),
        ],
        env: {
            let mut env = HashMap::new();
            env.insert("PYTHONPATH".to_string(), "/custom/python/path".to_string());
            env
        },
    };

    // Docker server
    let docker_config = ServerConfig {
        command: "docker".to_string(),
        args: vec![
            "run".to_string(),
            "-d".to_string(),
            "--name".to_string(),
            "mcp-container".to_string(),
            "-p".to_string(),
            "8080:8080".to_string(),
            "mcp/server:latest".to_string(),
        ],
        env: HashMap::new(),
    };

    // Binary server
    let binary_config = ServerConfig {
        command: "/usr/local/bin/mcp-server".to_string(),
        args: vec![
            "--port".to_string(),
            "9000".to_string(),
            "--workers".to_string(),
            "4".to_string(),
        ],
        env: {
            let mut env = HashMap::new();
            env.insert("LOG_LEVEL".to_string(), "debug".to_string());
            env
        },
    };

    // Verify each config is valid and different
    assert_ne!(npm_config, python_config);
    assert_ne!(python_config, docker_config);
    assert_ne!(docker_config, binary_config);
    
    // Verify serialization works for all types
    for config in &[npm_config, python_config, docker_config, binary_config] {
        let json = serde_json::to_string(config).unwrap();
        let deserialized: ServerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, &deserialized);
    }
}

#[test]
fn test_registry_operations_with_mixed_installation_states() {
    let mut registry = ClientRegistry::new();

    // Create a mix of installed and not installed clients
    // Since we can't easily mock the real clients' is_installed() method,
    // we'll use the actual implementations and just test the registry behavior

    let all_clients = detect_clients();
    for client in all_clients {
        registry.register(client);
    }

    // Get installed clients
    let installed = registry.detect_installed();
    
    // At least verify the method works and returns valid clients
    for client in installed {
        assert!(client.is_installed());
        assert!(!client.name().is_empty());
        assert!(!client.config_path().as_os_str().is_empty());
    }

    // Verify we can still look up clients by name regardless of installation status
    let all_names = vec!["Claude Code", "Claude Desktop", "Cursor", "VS Code", "Windsurf"];
    for name in all_names {
        assert!(
            registry.get_by_name(name).is_some(),
            "Should be able to find {name} by name"
        );
    }
}

#[test]
fn test_environment_variable_validation() {
    // Test various environment variable scenarios
    let test_cases = vec![
        // Valid cases
        (HashMap::from([("VALID_KEY".to_string(), "value".to_string())]), true),
        (HashMap::from([("PATH".to_string(), "/usr/bin:/bin".to_string())]), true),
        (HashMap::from([("_UNDERSCORE".to_string(), "ok".to_string())]), true),
        (HashMap::from([("NUM123".to_string(), "456".to_string())]), true),
        
        // Edge cases that should be handled
        (HashMap::from([("EMPTY_VALUE".to_string(), "".to_string())]), true),
        (HashMap::from([("UNICODE_KEY_🔑".to_string(), "value".to_string())]), true),
        
        // Multiple variables
        (HashMap::from([
            ("VAR1".to_string(), "value1".to_string()),
            ("VAR2".to_string(), "value2".to_string()),
            ("VAR3".to_string(), "value3".to_string()),
        ]), true),
    ];

    for (env, should_be_valid) in test_cases {
        let config = ServerConfig {
            command: "test".to_string(),
            args: vec![],
            env: env.clone(),
        };

        if should_be_valid {
            // Should serialize/deserialize without issues
            let json = serde_json::to_string(&config).unwrap();
            let _deserialized: ServerConfig = serde_json::from_str(&json).unwrap();
        }
    }
}