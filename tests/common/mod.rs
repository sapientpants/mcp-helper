#![allow(dead_code)]

use mcp_helper::client::{McpClient, ServerConfig};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// Creates a temporary directory with a config file
pub fn setup_test_config(initial_content: &str) -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");
    fs::write(&config_path, initial_content).unwrap();
    (temp_dir, config_path)
}

/// Creates a temporary directory with an empty JSON config
pub fn setup_empty_config() -> (TempDir, std::path::PathBuf) {
    setup_test_config("{}")
}

/// Reads and parses a JSON file
pub fn read_json_file(path: &Path) -> Value {
    let content = fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

/// Writes a JSON value to a file with pretty formatting
pub fn write_json_file(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

/// Common assertion helper for error messages
pub fn assert_error_contains(error_string: &str, expected_messages: &[&str]) {
    for msg in expected_messages {
        assert!(
            error_string.contains(msg),
            "Expected error to contain '{msg}', but got: {error_string}"
        );
    }
}

/// Creates a standard MCP server configuration
pub fn create_test_server_config(command: &str, args: Vec<&str>) -> Value {
    serde_json::json!({
        "command": command,
        "args": args.into_iter().map(|s| s.to_string()).collect::<Vec<_>>()
    })
}

/// Creates a config with mcpServers
pub fn create_config_with_servers(servers: serde_json::Map<String, Value>) -> Value {
    serde_json::json!({
        "mcpServers": servers
    })
}

/// Mock client for testing
pub struct MockClient {
    pub name: String,
    pub servers: Arc<Mutex<HashMap<String, ServerConfig>>>,
}

impl MockClient {
    /// Create a new mock client with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new mock client with pre-populated servers
    pub fn with_servers(name: impl Into<String>, servers: HashMap<String, ServerConfig>) -> Self {
        Self {
            name: name.into(),
            servers: Arc::new(Mutex::new(servers)),
        }
    }
}

impl McpClient for MockClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        PathBuf::from("/mock/config.json")
    }

    fn is_installed(&self) -> bool {
        true
    }

    fn add_server(&self, name: &str, config: ServerConfig) -> anyhow::Result<()> {
        self.servers
            .lock()
            .unwrap()
            .insert(name.to_string(), config);
        Ok(())
    }

    fn list_servers(&self) -> anyhow::Result<HashMap<String, ServerConfig>> {
        Ok(self.servers.lock().unwrap().clone())
    }
}

/// Create an isolated configuration manager for testing
/// Returns both the ConfigManager and the TempDir (to keep it alive)
pub fn create_isolated_config_manager() -> (mcp_helper::config::ConfigManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();

    // Set the environment variable for this test
    std::env::set_var("XDG_DATA_HOME", temp_dir.path());

    // Now create the manager - it will use the temp directory
    let manager = mcp_helper::config::ConfigManager::new().unwrap();

    (manager, temp_dir)
}
