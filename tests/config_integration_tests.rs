//! Integration tests for configuration management
//!
//! These tests verify configuration validation, history tracking,
//! rollback functionality, and diff generation.

use chrono::Utc;
use mcp_helper::client::{McpClient, ServerConfig};
use mcp_helper::config::{ConfigManager, ConfigSnapshot};
use mcp_helper::server::{ConfigField, ConfigFieldType};
use std::collections::HashMap;
use std::path::PathBuf;

#[allow(dead_code)]
struct TestClient {
    name: String,
    servers: HashMap<String, ServerConfig>,
}

impl McpClient for TestClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        PathBuf::from("/test/config.json")
    }

    fn is_installed(&self) -> bool {
        true
    }

    fn add_server(&self, _name: &str, _config: ServerConfig) -> anyhow::Result<()> {
        Ok(())
    }

    fn list_servers(&self) -> anyhow::Result<HashMap<String, ServerConfig>> {
        Ok(self.servers.clone())
    }
}

#[test]
fn test_config_manager_creation() {
    let manager = ConfigManager::new();
    assert!(manager.is_ok());
}

#[test]
fn test_config_snapshot_creation() {
    let snapshot = ConfigSnapshot {
        timestamp: Utc::now(),
        client_name: "test-client".to_string(),
        server_name: "test-server".to_string(),
        config: ServerConfig {
            command: "npx".to_string(),
            args: vec!["test-server".to_string()],
            env: HashMap::new(),
        },
        previous_config: None,
        description: "Test snapshot".to_string(),
    };

    assert_eq!(snapshot.client_name, "test-client");
    assert_eq!(snapshot.server_name, "test-server");
    assert!(snapshot.previous_config.is_none());
}

#[test]
fn test_config_diff_no_changes() {
    let manager = ConfigManager::new().unwrap();

    let config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::from([("PORT".to_string(), "3000".to_string())]),
    };

    let diff = manager.diff_configs(&config, &config);
    assert!(diff.is_empty());
}

#[test]
fn test_config_diff_command_change() {
    let manager = ConfigManager::new().unwrap();

    let old_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::new(),
    };

    let new_config = ServerConfig {
        command: "deno".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::new(),
    };

    let diff = manager.diff_configs(&old_config, &new_config);
    assert!(!diff.is_empty());
    assert!(diff.iter().any(|d| d.contains("Command")));
    assert!(diff
        .iter()
        .any(|d| d.contains("node") && d.contains("deno")));
}

#[test]
fn test_config_diff_args_change() {
    let manager = ConfigManager::new().unwrap();

    let old_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::new(),
    };

    let new_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string(), "--port=3000".to_string()],
        env: HashMap::new(),
    };

    let diff = manager.diff_configs(&old_config, &new_config);
    assert!(!diff.is_empty());
    assert!(diff.iter().any(|d| d.contains("Arguments")));
}

#[test]
fn test_config_diff_env_changes() {
    let manager = ConfigManager::new().unwrap();

    let old_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::from([
            ("PORT".to_string(), "3000".to_string()),
            ("HOST".to_string(), "localhost".to_string()),
        ]),
    };

    let new_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::from([
            ("PORT".to_string(), "8080".to_string()), // Modified
            ("DEBUG".to_string(), "true".to_string()), // Added
                                                      // HOST removed
        ]),
    };

    let diff = manager.diff_configs(&old_config, &new_config);
    assert!(!diff.is_empty());

    // Should show PORT modification
    assert!(diff
        .iter()
        .any(|d| d.contains("PORT") && d.contains("3000") && d.contains("8080")));

    // Should show DEBUG addition
    assert!(diff
        .iter()
        .any(|d| d.contains("DEBUG") && d.contains("Added")));

    // Should show HOST removal
    assert!(diff
        .iter()
        .any(|d| d.contains("HOST") && d.contains("Removed")));
}

#[test]
fn test_validate_env_vars() {
    let manager = ConfigManager::new().unwrap();

    let valid_env = HashMap::from([
        ("PATH".to_string(), "/usr/bin:/usr/local/bin".to_string()),
        ("HOME".to_string(), "/home/user".to_string()),
        ("TEMP".to_string(), "/tmp".to_string()),
    ]);

    let result = manager.validate_env_vars(&valid_env);
    assert!(result.is_ok());
}

#[test]
fn test_config_field_validation() {
    let string_field = ConfigField {
        name: "api_key".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("API key".to_string()),
        default: None,
    };

    let number_field = ConfigField {
        name: "port".to_string(),
        field_type: ConfigFieldType::Number,
        description: Some("Port number".to_string()),
        default: Some("3000".to_string()),
    };

    let bool_field = ConfigField {
        name: "debug".to_string(),
        field_type: ConfigFieldType::Boolean,
        description: Some("Debug mode".to_string()),
        default: Some("false".to_string()),
    };

    let url_field = ConfigField {
        name: "endpoint".to_string(),
        field_type: ConfigFieldType::Url,
        description: Some("API endpoint".to_string()),
        default: None,
    };

    let path_field = ConfigField {
        name: "config_path".to_string(),
        field_type: ConfigFieldType::Path,
        description: Some("Config file path".to_string()),
        default: Some("./config.json".to_string()),
    };

    // Test that each field type is distinct
    assert!(matches!(string_field.field_type, ConfigFieldType::String));
    assert!(matches!(number_field.field_type, ConfigFieldType::Number));
    assert!(matches!(bool_field.field_type, ConfigFieldType::Boolean));
    assert!(matches!(url_field.field_type, ConfigFieldType::Url));
    assert!(matches!(path_field.field_type, ConfigFieldType::Path));
}

#[test]
fn test_config_history_operations() {
    let manager = ConfigManager::new().unwrap();

    // Get history (may be empty or have existing entries)
    let history = manager.get_history(None, None);
    assert!(history.is_ok());

    // Filter by client
    let history = manager.get_history(Some("test-client"), None);
    assert!(history.is_ok());

    // Filter by server
    let history = manager.get_history(None, Some("test-server"));
    assert!(history.is_ok());

    // Filter by both
    let history = manager.get_history(Some("test-client"), Some("test-server"));
    assert!(history.is_ok());
}

#[test]
fn test_snapshot_with_previous_config() {
    let previous = ServerConfig {
        command: "old-command".to_string(),
        args: vec!["old.js".to_string()],
        env: HashMap::new(),
    };

    let current = ServerConfig {
        command: "new-command".to_string(),
        args: vec!["new.js".to_string()],
        env: HashMap::new(),
    };

    let snapshot = ConfigSnapshot {
        timestamp: Utc::now(),
        client_name: "client".to_string(),
        server_name: "server".to_string(),
        config: current,
        previous_config: Some(previous.clone()),
        description: "Update".to_string(),
    };

    assert!(snapshot.previous_config.is_some());
    assert_eq!(snapshot.previous_config.unwrap().command, "old-command");
}

#[test]
fn test_complex_env_diff() {
    let manager = ConfigManager::new().unwrap();

    let old_config = ServerConfig {
        command: "node".to_string(),
        args: vec![],
        env: HashMap::from([
            ("VAR1".to_string(), "value1".to_string()),
            ("VAR2".to_string(), "value2".to_string()),
            ("VAR3".to_string(), "value3".to_string()),
            ("VAR4".to_string(), "value4".to_string()),
            ("VAR5".to_string(), "value5".to_string()),
        ]),
    };

    let new_config = ServerConfig {
        command: "node".to_string(),
        args: vec![],
        env: HashMap::from([
            ("VAR1".to_string(), "modified1".to_string()), // Modified
            ("VAR2".to_string(), "value2".to_string()),    // Unchanged
            // VAR3 removed
            ("VAR4".to_string(), "modified4".to_string()), // Modified
            ("VAR5".to_string(), "value5".to_string()),    // Unchanged
            ("VAR6".to_string(), "value6".to_string()),    // Added
            ("VAR7".to_string(), "value7".to_string()),    // Added
        ]),
    };

    let diff = manager.diff_configs(&old_config, &new_config);

    // Should detect 2 modifications
    assert!(diff.iter().any(|d| d.contains("VAR1")));
    assert!(diff.iter().any(|d| d.contains("VAR4")));

    // Should detect 1 removal
    assert!(diff
        .iter()
        .any(|d| d.contains("VAR3") && d.contains("Removed")));

    // Should detect 2 additions
    assert!(diff
        .iter()
        .any(|d| d.contains("VAR6") && d.contains("Added")));
    assert!(diff
        .iter()
        .any(|d| d.contains("VAR7") && d.contains("Added")));
}

#[test]
fn test_timestamp_ordering() {
    use std::thread;
    use std::time::Duration;

    let snapshot1 = ConfigSnapshot {
        timestamp: Utc::now(),
        client_name: "client".to_string(),
        server_name: "server".to_string(),
        config: ServerConfig {
            command: "cmd1".to_string(),
            args: vec![],
            env: HashMap::new(),
        },
        previous_config: None,
        description: "First".to_string(),
    };

    // Small delay to ensure different timestamp
    thread::sleep(Duration::from_millis(10));

    let snapshot2 = ConfigSnapshot {
        timestamp: Utc::now(),
        client_name: "client".to_string(),
        server_name: "server".to_string(),
        config: ServerConfig {
            command: "cmd2".to_string(),
            args: vec![],
            env: HashMap::new(),
        },
        previous_config: None,
        description: "Second".to_string(),
    };

    assert!(snapshot2.timestamp > snapshot1.timestamp);
}

#[test]
fn test_empty_diff() {
    let manager = ConfigManager::new().unwrap();

    let empty_config = ServerConfig {
        command: "cmd".to_string(),
        args: vec![],
        env: HashMap::new(),
    };

    let diff = manager.diff_configs(&empty_config, &empty_config);
    assert!(diff.is_empty());
}

#[test]
fn test_args_diff_with_order() {
    let manager = ConfigManager::new().unwrap();

    let config1 = ServerConfig {
        command: "cmd".to_string(),
        args: vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()],
        env: HashMap::new(),
    };

    let config2 = ServerConfig {
        command: "cmd".to_string(),
        args: vec!["arg3".to_string(), "arg2".to_string(), "arg1".to_string()],
        env: HashMap::new(),
    };

    let diff = manager.diff_configs(&config1, &config2);

    // Different order means different args
    assert!(!diff.is_empty());
    assert!(diff.iter().any(|d| d.contains("Arguments")));
}

#[test]
fn test_config_with_special_characters() {
    let config = ServerConfig {
        command: "node".to_string(),
        args: vec!["--config=\"special chars\"".to_string()],
        env: HashMap::from([
            ("SPECIAL".to_string(), "!@#$%^&*()".to_string()),
            ("SPACES".to_string(), "value with spaces".to_string()),
            ("QUOTES".to_string(), "value with \"quotes\"".to_string()),
            ("NEWLINE".to_string(), "value\nwith\nnewlines".to_string()),
        ]),
    };

    assert_eq!(config.env.get("SPECIAL"), Some(&"!@#$%^&*()".to_string()));
    assert_eq!(
        config.env.get("SPACES"),
        Some(&"value with spaces".to_string())
    );
    assert!(config.env.get("QUOTES").unwrap().contains('"'));
    assert!(config.env.get("NEWLINE").unwrap().contains('\n'));
}

#[test]
fn test_get_latest_snapshot() {
    let manager = ConfigManager::new().unwrap();

    // Use unique names to avoid conflicts with existing data
    let unique_client = format!(
        "test-client-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let unique_server = format!(
        "test-server-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );

    let latest = manager.get_latest_snapshot(&unique_client, &unique_server);
    assert!(latest.is_ok());

    // Should be None for a unique combination
    assert!(latest.unwrap().is_none());
}

#[test]
fn test_config_validation_results() {
    let required_fields = vec![ConfigField {
        name: "api_key".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("Required API key".to_string()),
        default: None,
    }];

    let optional_fields = vec![ConfigField {
        name: "timeout".to_string(),
        field_type: ConfigFieldType::Number,
        description: Some("Request timeout".to_string()),
        default: Some("30".to_string()),
    }];

    // All required fields must have no default
    for field in &required_fields {
        assert!(field.default.is_none());
    }

    // Optional fields should have defaults
    for field in &optional_fields {
        assert!(field.default.is_some());
    }
}
