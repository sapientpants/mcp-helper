//! Real implementation tests for MCP clients
//!
//! This test suite uses real client implementations instead of mocks,
//! ensuring we're testing actual behavior.

use mcp_helper::client::{detect_clients, ClientRegistry, ServerConfig};
use std::collections::HashMap;

#[test]
fn test_server_config_creation() {
    let mut env = HashMap::new();
    env.insert("KEY".to_string(), "VALUE".to_string());

    let config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env,
    };

    assert_eq!(config.command, "node");
    assert_eq!(config.args, vec!["server.js"]);
    assert_eq!(config.env.get("KEY"), Some(&"VALUE".to_string()));
}

#[test]
fn test_server_config_with_complex_env() {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/local/bin:/usr/bin".to_string());
    env.insert("NODE_ENV".to_string(), "production".to_string());
    env.insert("PORT".to_string(), "3000".to_string());
    env.insert("DEBUG".to_string(), "app:*".to_string());

    let config = ServerConfig {
        command: "node".to_string(),
        args: vec![
            "server.js".to_string(),
            "--config".to_string(),
            "production.json".to_string(),
        ],
        env: env.clone(),
    };

    assert_eq!(config.command, "node");
    assert_eq!(config.args.len(), 3);
    assert_eq!(config.env.len(), 4);
    assert_eq!(config.env.get("NODE_ENV"), Some(&"production".to_string()));
}

#[test]
fn test_server_config_empty() {
    let config = ServerConfig {
        command: "echo".to_string(),
        args: vec![],
        env: HashMap::new(),
    };

    assert_eq!(config.command, "echo");
    assert!(config.args.is_empty());
    assert!(config.env.is_empty());
}

#[test]
fn test_server_config_clone() {
    let config = ServerConfig {
        command: "python".to_string(),
        args: vec!["app.py".to_string(), "--port=8080".to_string()],
        env: HashMap::from([("PYTHONPATH".to_string(), "/usr/lib/python".to_string())]),
    };

    let cloned = config.clone();
    assert_eq!(cloned.command, config.command);
    assert_eq!(cloned.args, config.args);
    assert_eq!(cloned.env, config.env);
}

#[test]
fn test_client_registry_new() {
    let registry = ClientRegistry::new();
    assert_eq!(registry.clients.len(), 0);
}

#[test]
fn test_client_registry_default() {
    let registry = ClientRegistry::default();
    assert_eq!(registry.clients.len(), 0);
}

#[test]
fn test_detect_clients_returns_all_types() {
    let clients = detect_clients();

    // Should return all registered client types
    assert!(clients.len() >= 5); // We have at least 5 client types

    let names: Vec<&str> = clients.iter().map(|c| c.name()).collect();
    assert!(names.contains(&"Claude Code"));
    assert!(names.contains(&"Claude Desktop"));
    assert!(names.contains(&"Cursor"));
    assert!(names.contains(&"VS Code"));
    assert!(names.contains(&"Windsurf"));
}

#[test]
fn test_detect_clients_unique_names() {
    let clients = detect_clients();

    let names: Vec<&str> = clients.iter().map(|c| c.name()).collect();
    let unique_names: std::collections::HashSet<&str> = names.iter().cloned().collect();

    // All names should be unique
    assert_eq!(names.len(), unique_names.len());
}

#[test]
fn test_client_registry_with_real_clients() {
    let mut registry = ClientRegistry::new();

    // Get real clients from detect_clients
    for client in detect_clients() {
        registry.register(client);
    }

    assert!(registry.clients.len() >= 5);

    // Test get_by_name with real client names
    assert!(registry.get_by_name("Claude Desktop").is_some());
    assert!(registry.get_by_name("claude desktop").is_some()); // Case insensitive
    assert!(registry.get_by_name("CLAUDE DESKTOP").is_some()); // Case insensitive

    assert!(registry.get_by_name("Cursor").is_some());
    assert!(registry.get_by_name("cursor").is_some());

    assert!(registry.get_by_name("VS Code").is_some());
    assert!(registry.get_by_name("Windsurf").is_some());
    assert!(registry.get_by_name("Claude Code").is_some());

    assert!(registry.get_by_name("NonExistent").is_none());
}

#[test]
fn test_client_registry_detect_installed() {
    let mut registry = ClientRegistry::new();

    // Add all real clients
    for client in detect_clients() {
        registry.register(client);
    }

    // detect_installed should return only those that are actually installed
    let installed = registry.detect_installed();

    // The number of installed clients will vary by system
    // but the result should be valid
    for client in &installed {
        assert!(client.is_installed());
    }

    // Verify that all returned clients have valid names
    for client in &installed {
        assert!(!client.name().is_empty());
    }
}

#[test]
fn test_client_registry_register_multiple() {
    let mut registry = ClientRegistry::new();

    let initial_count = registry.clients.len();

    // Register multiple clients
    for client in detect_clients().into_iter().take(3) {
        registry.register(client);
    }

    assert_eq!(registry.clients.len(), initial_count + 3);
}

#[test]
fn test_server_config_with_many_args() {
    let config = ServerConfig {
        command: "docker".to_string(),
        args: vec![
            "run".to_string(),
            "--rm".to_string(),
            "-it".to_string(),
            "--name".to_string(),
            "mcp-server".to_string(),
            "-p".to_string(),
            "3000:3000".to_string(),
            "-v".to_string(),
            "/data:/data".to_string(),
            "mcp-server:latest".to_string(),
        ],
        env: HashMap::new(),
    };

    assert_eq!(config.command, "docker");
    assert_eq!(config.args.len(), 10);
    assert_eq!(config.args[0], "run");
    assert_eq!(config.args[9], "mcp-server:latest");
}

#[test]
fn test_server_config_with_special_chars_in_env() {
    let mut env = HashMap::new();
    env.insert("SPECIAL_CHARS".to_string(), "!@#$%^&*()".to_string());
    env.insert("SPACES".to_string(), "value with spaces".to_string());
    env.insert("EMPTY".to_string(), "".to_string());
    env.insert(
        "PATH_WITH_COLON".to_string(),
        "/usr/bin:/usr/local/bin".to_string(),
    );

    let config = ServerConfig {
        command: "test".to_string(),
        args: vec![],
        env: env.clone(),
    };

    assert_eq!(
        config.env.get("SPECIAL_CHARS"),
        Some(&"!@#$%^&*()".to_string())
    );
    assert_eq!(
        config.env.get("SPACES"),
        Some(&"value with spaces".to_string())
    );
    assert_eq!(config.env.get("EMPTY"), Some(&"".to_string()));
    assert_eq!(
        config.env.get("PATH_WITH_COLON"),
        Some(&"/usr/bin:/usr/local/bin".to_string())
    );
}

#[test]
fn test_client_names_consistency() {
    let clients = detect_clients();

    // Check that client names follow expected patterns
    for client in clients {
        let name = client.name();

        // Name should not be empty
        assert!(!name.is_empty());

        // Name should not have leading/trailing whitespace
        assert_eq!(name, name.trim());

        // Check for expected client names
        assert!(
            name == "Claude Desktop"
                || name == "Claude Code"
                || name == "Cursor"
                || name == "VS Code"
                || name == "Windsurf",
            "Unexpected client name: {name}"
        );
    }
}

#[test]
fn test_client_config_paths_valid() {
    let clients = detect_clients();

    for client in clients {
        let path = client.config_path();

        // Path should not be empty
        assert!(!path.as_os_str().is_empty());

        // Path should be absolute or contain expected components
        let path_str = path.to_string_lossy();

        // Check for expected path patterns
        assert!(
            path_str.contains("claude")
                || path_str.contains("Claude")
                || path_str.contains("cursor")
                || path_str.contains("Cursor")
                || path_str.contains("Code")
                || path_str.contains("Windsurf")
                || path_str.contains("windsurf")
                || path_str.contains("vscode")
                || path_str.contains(".vscode")
                || path_str.contains("settings.json")
                || path_str.contains("mcp.json"),
            "Unexpected config path: {path_str}"
        );
    }
}

#[test]
fn test_registry_operations_sequence() {
    let mut registry = ClientRegistry::new();

    // Initially empty
    assert_eq!(registry.clients.len(), 0);
    assert!(registry.detect_installed().is_empty());
    assert!(registry.get_by_name("anything").is_none());

    // Add one client
    if let Some(first_client) = detect_clients().into_iter().next() {
        let client_name = first_client.name().to_string();
        registry.register(first_client);

        assert_eq!(registry.clients.len(), 1);
        assert!(registry.get_by_name(&client_name).is_some());

        // Case insensitive lookup
        assert!(registry.get_by_name(&client_name.to_lowercase()).is_some());
        assert!(registry.get_by_name(&client_name.to_uppercase()).is_some());
    }
}

#[test]
fn test_server_config_equality() {
    let config1 = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::from([("PORT".to_string(), "3000".to_string())]),
    };

    let config2 = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::from([("PORT".to_string(), "3000".to_string())]),
    };

    let config3 = ServerConfig {
        command: "deno".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::from([("PORT".to_string(), "3000".to_string())]),
    };

    // Same configs should be equal
    assert_eq!(config1.command, config2.command);
    assert_eq!(config1.args, config2.args);
    assert_eq!(config1.env, config2.env);

    // Different command
    assert_ne!(config1.command, config3.command);
}
