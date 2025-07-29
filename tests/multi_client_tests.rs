// NOTE: These tests modify the HOME environment variable and must be run with --test-threads=1
// to avoid conflicts between parallel test execution.

use mcp_helper::client::{
    ClaudeCodeClient, ClaudeDesktopClient, ClientRegistry, CursorClient, McpClient, ServerConfig,
    VSCodeClient, WindsurfClient,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_all_clients_registered() {
    let clients = mcp_helper::client::detect_clients();
    assert_eq!(clients.len(), 5);

    let names: Vec<&str> = clients.iter().map(|c| c.name()).collect();
    assert!(names.contains(&"Claude Code"));
    assert!(names.contains(&"Claude Desktop"));
    assert!(names.contains(&"Cursor"));
    assert!(names.contains(&"VS Code"));
    assert!(names.contains(&"Windsurf"));
}

#[test]
fn test_client_registry_get_by_name() {
    let mut registry = ClientRegistry::new();
    registry.register(Box::new(ClaudeCodeClient::new()));
    registry.register(Box::new(CursorClient::new()));
    registry.register(Box::new(VSCodeClient::new()));
    registry.register(Box::new(WindsurfClient::new()));

    // Test exact match
    assert!(registry.get_by_name("Claude Code").is_some());
    assert!(registry.get_by_name("Cursor").is_some());
    assert!(registry.get_by_name("VS Code").is_some());
    assert!(registry.get_by_name("Windsurf").is_some());

    // Test case insensitive
    assert!(registry.get_by_name("claude code").is_some());
    assert!(registry.get_by_name("CLAUDE CODE").is_some());
    assert!(registry.get_by_name("cursor").is_some());
    assert!(registry.get_by_name("CURSOR").is_some());
    assert!(registry.get_by_name("vs code").is_some());
    assert!(registry.get_by_name("windsurf").is_some());

    // Test not found
    assert!(registry.get_by_name("Unknown").is_none());
}

#[test]
fn test_multiple_client_installation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_path_buf();

    // Override HOME for test
    // Save original HOME/USERPROFILE for restoration
    let original_home = if cfg!(windows) {
        env::var("USERPROFILE").ok()
    } else {
        env::var("HOME").ok()
    };

    // Set appropriate home directory variable
    if cfg!(windows) {
        env::set_var("USERPROFILE", &temp_home);
    } else {
        env::set_var("HOME", &temp_home);
    }

    let clients: Vec<Box<dyn McpClient>> = vec![
        Box::new(CursorClient::new()),
        Box::new(VSCodeClient::new()),
        Box::new(WindsurfClient::new()),
    ];

    let config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::new(),
    };

    // Install to all clients
    for client in &clients {
        println!(
            "Installing to client: {} at path: {:?}",
            client.name(),
            client.config_path()
        );
        let result = client.add_server("test-server", config.clone());
        assert!(
            result.is_ok(),
            "Failed to install to {}: {:?}",
            client.name(),
            result
        );
    }

    // Verify all configs were created
    let cursor_path = temp_home.join(".cursor").join("mcp.json");
    let vscode_path = temp_home.join(".vscode").join("mcp.json");
    let windsurf_path = temp_home
        .join(".codeium")
        .join("windsurf")
        .join("mcp_config.json");

    println!("Checking paths:");
    println!(
        "  Cursor: {:?} exists: {}",
        cursor_path,
        cursor_path.exists()
    );
    println!(
        "  VS Code: {:?} exists: {}",
        vscode_path,
        vscode_path.exists()
    );
    println!(
        "  Windsurf: {:?} exists: {}",
        windsurf_path,
        windsurf_path.exists()
    );

    assert!(cursor_path.exists(), "Cursor config should exist");
    assert!(vscode_path.exists(), "VS Code config should exist");
    assert!(windsurf_path.exists(), "Windsurf config should exist");

    // Verify all can list the server
    for client in &clients {
        let servers = client.list_servers().unwrap();
        assert_eq!(
            servers.len(),
            1,
            "Client {} should have 1 server",
            client.name()
        );
        assert!(
            servers.contains_key("test-server"),
            "Client {} should have test-server",
            client.name()
        );
    }

    // Restore HOME
    // Restore original HOME/USERPROFILE
    match original_home {
        Some(home) => {
            if cfg!(windows) {
                env::set_var("USERPROFILE", home);
            } else {
                env::set_var("HOME", home);
            }
        }
        None => {
            if cfg!(windows) {
                env::remove_var("USERPROFILE");
            } else {
                env::remove_var("HOME");
            }
        }
    }
}

#[test]
fn test_client_config_formats() {
    // Suppress VS Code warnings to stderr
    let _stderr = std::io::stderr();
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_path_buf();

    // Save original HOME/USERPROFILE for restoration
    let original_home = if cfg!(windows) {
        env::var("USERPROFILE").ok()
    } else {
        env::var("HOME").ok()
    };

    // Set appropriate home directory variable
    if cfg!(windows) {
        env::set_var("USERPROFILE", &temp_home);
    } else {
        env::set_var("HOME", &temp_home);
    }

    // Test Cursor format
    let cursor = CursorClient::new();
    cursor
        .add_server(
            "test",
            ServerConfig {
                command: "cmd".to_string(),
                args: vec!["arg".to_string()],
                env: HashMap::new(),
            },
        )
        .unwrap();

    let cursor_content = fs::read_to_string(temp_home.join(".cursor").join("mcp.json")).unwrap();
    assert!(cursor_content.contains("\"type\": \"stdio\""));

    // Test VS Code format (same as Cursor)
    let vscode = VSCodeClient::new();
    vscode
        .add_server(
            "test",
            ServerConfig {
                command: "cmd".to_string(),
                args: vec!["arg".to_string()],
                env: HashMap::new(),
            },
        )
        .unwrap();

    let vscode_path = temp_home.join(".vscode").join("mcp.json");
    assert!(
        vscode_path.exists(),
        "VS Code config should exist at {vscode_path:?}"
    );
    let vscode_content = fs::read_to_string(vscode_path).unwrap();
    assert!(vscode_content.contains("\"type\": \"stdio\""));

    // Test Windsurf format
    let windsurf = WindsurfClient::new();
    windsurf
        .add_server(
            "test",
            ServerConfig {
                command: "cmd".to_string(),
                args: vec!["arg".to_string()],
                env: HashMap::new(),
            },
        )
        .unwrap();

    let windsurf_content = fs::read_to_string(
        temp_home
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json"),
    )
    .unwrap();
    assert!(windsurf_content.contains("\"mcpServers\""));

    // Restore original HOME/USERPROFILE
    match original_home {
        Some(home) => {
            if cfg!(windows) {
                env::set_var("USERPROFILE", home);
            } else {
                env::set_var("HOME", home);
            }
        }
        None => {
            if cfg!(windows) {
                env::remove_var("USERPROFILE");
            } else {
                env::remove_var("HOME");
            }
        }
    }
}

#[test]
fn test_detect_installed_clients() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_path_buf();

    // Save original HOME/USERPROFILE for restoration
    let original_home = if cfg!(windows) {
        env::var("USERPROFILE").ok()
    } else {
        env::var("HOME").ok()
    };

    // Set appropriate home directory variable
    if cfg!(windows) {
        env::set_var("USERPROFILE", &temp_home);
    } else {
        env::set_var("HOME", &temp_home);
    }

    // Create config directories for some clients
    fs::create_dir_all(temp_home.join(".cursor")).unwrap();
    fs::create_dir_all(temp_home.join(".codeium").join("windsurf")).unwrap();

    // For Claude Desktop on macOS in tests
    #[cfg(target_os = "macos")]
    fs::create_dir_all(
        temp_home
            .join("Library")
            .join("Application Support")
            .join("Claude"),
    )
    .unwrap();

    let mut registry = ClientRegistry::new();
    registry.register(Box::new(ClaudeDesktopClient::new()));
    registry.register(Box::new(CursorClient::new()));
    registry.register(Box::new(VSCodeClient::new()));
    registry.register(Box::new(WindsurfClient::new()));

    let installed = registry.detect_installed();
    #[cfg(target_os = "macos")]
    assert_eq!(installed.len(), 3);
    #[cfg(not(target_os = "macos"))]
    assert_eq!(installed.len(), 2);

    let names: Vec<&str> = installed.iter().map(|c| c.name()).collect();
    assert!(names.contains(&"Cursor"));
    assert!(names.contains(&"Windsurf"));
    assert!(!names.contains(&"VS Code"));

    // Restore original HOME/USERPROFILE
    match original_home {
        Some(home) => {
            if cfg!(windows) {
                env::set_var("USERPROFILE", home);
            } else {
                env::set_var("HOME", home);
            }
        }
        None => {
            if cfg!(windows) {
                env::remove_var("USERPROFILE");
            } else {
                env::remove_var("HOME");
            }
        }
    }
}

#[test]
fn test_client_with_env_vars() {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().to_path_buf();

    // Save original HOME/USERPROFILE for restoration
    let original_home = if cfg!(windows) {
        env::var("USERPROFILE").ok()
    } else {
        env::var("HOME").ok()
    };

    // Set appropriate home directory variable
    if cfg!(windows) {
        env::set_var("USERPROFILE", &temp_home);
    } else {
        env::set_var("HOME", &temp_home);
    }

    let mut env = HashMap::new();
    env.insert("API_KEY".to_string(), "secret-key".to_string());
    env.insert("DEBUG".to_string(), "true".to_string());

    let config = ServerConfig {
        command: "npx".to_string(),
        args: vec!["server".to_string()],
        env: env.clone(),
    };

    // Test all clients handle env vars
    let clients: Vec<Box<dyn McpClient>> = vec![
        Box::new(ClaudeCodeClient::new()),
        Box::new(CursorClient::new()),
        Box::new(VSCodeClient::new()),
        Box::new(WindsurfClient::new()),
    ];

    for client in &clients {
        client.add_server("env-test", config.clone()).unwrap();

        let servers = client.list_servers().unwrap();
        let server = &servers["env-test"];
        assert_eq!(server.env.len(), 2);
        assert_eq!(server.env.get("API_KEY"), Some(&"secret-key".to_string()));
        assert_eq!(server.env.get("DEBUG"), Some(&"true".to_string()));
    }

    // Restore original HOME/USERPROFILE
    match original_home {
        Some(home) => {
            if cfg!(windows) {
                env::set_var("USERPROFILE", home);
            } else {
                env::set_var("HOME", home);
            }
        }
        None => {
            if cfg!(windows) {
                env::remove_var("USERPROFILE");
            } else {
                env::remove_var("HOME");
            }
        }
    }
}
