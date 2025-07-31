mod common;
use common::{create_isolated_config_manager, MockClient};

use mcp_helper::client::{McpClient, ServerConfig};
use std::collections::HashMap;

#[test]
fn test_simple_config_snapshot_creation() {
    let (manager, _temp_dir) = create_isolated_config_manager();

    let client = MockClient::new("test-client");

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
fn test_simple_config_rollback() {
    let (manager, _temp_dir) = create_isolated_config_manager();

    let client = MockClient::new("rollback-client");

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
fn test_simple_config_history() {
    let (manager, _temp_dir) = create_isolated_config_manager();

    let client = MockClient::new("history-client");

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
fn test_simple_config_diff() {
    let (manager, _temp_dir) = create_isolated_config_manager();

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
    assert!(diffs
        .iter()
        .any(|d| d.contains("Added env var: PRODUCTION=true")));
    assert!(diffs.iter().any(|d| d.contains("Removed env var: DEBUG")));
}

#[test]
fn test_simple_latest_snapshot() {
    let (manager, _temp_dir) = create_isolated_config_manager();

    let client = MockClient::new("latest-client");

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
