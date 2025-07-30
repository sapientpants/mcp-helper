use mcp_helper::client::{McpClient, ServerConfig};
use mcp_helper::config::ConfigManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

// Mock client for testing
struct MockClient {
    name: String,
    servers: Arc<Mutex<HashMap<String, ServerConfig>>>,
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

#[test]
fn test_config_snapshot_creation() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join(format!("snapshot_test_{}", std::process::id()));
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let manager = ConfigManager::new().unwrap();

    let client = MockClient {
        name: "test-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
    };

    let config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::new(),
    };

    // Apply configuration and create snapshot
    let snapshot = manager
        .apply_config(&client, "test-server", config.clone())
        .unwrap();

    // Verify snapshot was created
    assert_eq!(snapshot.server_name, "test-server");
    assert_eq!(snapshot.client_name, "test-client");
    assert_eq!(snapshot.config.command, "node");
    assert!(snapshot.previous_config.is_none()); // First installation

    // Verify server was added to client
    let servers = client.list_servers().unwrap();
    assert!(servers.contains_key("test-server"));
}

#[test]
fn test_config_rollback_with_previous_config() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join(format!("rollback_test_{}", std::process::id()));
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let manager = ConfigManager::new().unwrap();

    let client = MockClient {
        name: "rollback-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
    };

    // Install initial configuration
    let initial_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::new(),
    };

    manager
        .apply_config(&client, "test-server", initial_config.clone())
        .unwrap();

    // Update configuration
    let mut updated_config = initial_config.clone();
    updated_config.args.push("--port=3000".to_string());

    let update_snapshot = manager
        .apply_config(&client, "test-server", updated_config.clone())
        .unwrap();

    // Verify updated config is active
    let servers = client.list_servers().unwrap();
    assert_eq!(servers["test-server"].args.len(), 2);

    // Rollback to previous configuration
    manager.rollback(&client, &update_snapshot).unwrap();

    // Verify rollback worked
    let servers = client.list_servers().unwrap();
    assert_eq!(servers["test-server"].args.len(), 1);
    assert_eq!(servers["test-server"].args[0], "server.js");
}

#[test]
fn test_config_rollback_without_previous_config() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join(format!("no_previous_test_{}", std::process::id()));
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let manager = ConfigManager::new().unwrap();

    let client = MockClient {
        name: "no-previous-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
    };

    let config = ServerConfig {
        command: "deno".to_string(),
        args: vec!["run", "server.ts"].iter().map(|s| s.to_string()).collect(),
        env: HashMap::new(),
    };

    let snapshot = manager
        .apply_config(&client, "new-server", config.clone())
        .unwrap();

    // This should fail because there's no previous config to rollback to
    let result = manager.rollback(&client, &snapshot);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no previous configuration"));
}

#[test]
fn test_config_history_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join(format!("history_test_{}", std::process::id()));
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let manager = ConfigManager::new().unwrap();

    let client = MockClient {
        name: "history-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
    };

    // Apply multiple configurations
    for i in 0..3 {
        let config = ServerConfig {
            command: "node".to_string(),
            args: vec![format!("server-{}.js", i)],
            env: HashMap::new(),
        };

        manager
            .apply_config(&client, "tracked-server", config)
            .unwrap();
    }

    // Check history
    let history = manager
        .get_history(Some("history-client"), Some("tracked-server"))
        .unwrap();

    assert_eq!(history.len(), 3);
    
    // History should be sorted by timestamp (newest first)
    for i in 0..history.len() - 1 {
        assert!(history[i].timestamp >= history[i + 1].timestamp);
    }
}

#[test]
fn test_config_diff_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join(format!("diff_test_{}", std::process::id()));
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let manager = ConfigManager::new().unwrap();

    let old_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["old-server.js".to_string()],
        env: {
            let mut env = HashMap::new();
            env.insert("PORT".to_string(), "3000".to_string());
            env.insert("DEBUG".to_string(), "false".to_string());
            env
        },
    };

    let new_config = ServerConfig {
        command: "deno".to_string(),
        args: vec!["new-server.ts".to_string(), "--allow-net".to_string()],
        env: {
            let mut env = HashMap::new();
            env.insert("PORT".to_string(), "4000".to_string());
            env.insert("PRODUCTION".to_string(), "true".to_string());
            env
        },
    };

    let diffs = manager.diff_configs(&old_config, &new_config);

    // Should detect command change
    assert!(diffs.iter().any(|d| d.contains("Command: node → deno")));
    
    // Should detect args change
    assert!(diffs.iter().any(|d| d.contains("Arguments:")));
    
    // Should detect env var changes
    assert!(diffs.iter().any(|d| d.contains("Modified env var PORT")));
    assert!(diffs.iter().any(|d| d.contains("Added env var: PRODUCTION=true")));
    assert!(diffs.iter().any(|d| d.contains("Removed env var: DEBUG")));
}

#[test]
fn test_config_history_cleanup() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join(format!("cleanup_test_{}", std::process::id()));
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let manager = ConfigManager::new().unwrap();

    let client = MockClient {
        name: "cleanup-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
    };

    // Apply more configurations than the max history limit (default is 10)
    for i in 0..15 {
        let config = ServerConfig {
            command: "node".to_string(),
            args: vec![format!("server-{}.js", i)],
            env: HashMap::new(),
        };

        manager
            .apply_config(&client, "cleanup-server", config)
            .unwrap();
    }

    // Check that history was cleaned up
    let history = manager
        .get_history(Some("cleanup-client"), Some("cleanup-server"))
        .unwrap();

    // Should not exceed max history entries (10)
    assert!(history.len() <= 10);
    
    // Should keep the most recent entries
    // The newest entry should have the highest server number
    if let Some(newest) = history.first() {
        assert!(newest.config.args[0].contains("server-14.js"));
    }
}

#[test]
fn test_latest_snapshot_retrieval() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join(format!("latest_test_{}", std::process::id()));
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let manager = ConfigManager::new().unwrap();

    let client = MockClient {
        name: "latest-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
    };

    // Apply a few configurations
    for i in 0..3 {
        let config = ServerConfig {
            command: "node".to_string(),
            args: vec![format!("version-{}.js", i)],
            env: HashMap::new(),
        };

        manager
            .apply_config(&client, "latest-server", config)
            .unwrap();
    }

    // Get latest snapshot
    let latest = manager
        .get_latest_snapshot("latest-client", "latest-server")
        .unwrap();

    assert!(latest.is_some());
    let snapshot = latest.unwrap();
    assert!(snapshot.config.args[0].contains("version-2.js"));
    assert_eq!(snapshot.server_name, "latest-server");
    assert_eq!(snapshot.client_name, "latest-client");
}