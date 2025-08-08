mod common;
use common::create_isolated_config_manager;
use mcp_helper::test_utils::mocks::MockClientBuilder;

use mcp_helper::client::{McpClient, ServerConfig};
use serial_test::serial;
use std::collections::HashMap;

#[test]
#[serial]
fn test_simple_config_snapshot_creation() {
    let (manager, temp_dir) = create_isolated_config_manager();

    let client = MockClientBuilder::new("test-client").build();

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

    // Clean up
    std::env::remove_var("XDG_DATA_HOME");
    drop(temp_dir);
}

#[test]
#[serial]
fn test_simple_config_rollback() {
    let (manager, temp_dir) = create_isolated_config_manager();

    let client = MockClientBuilder::new("rollback-client").build();

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

    // Clean up
    std::env::remove_var("XDG_DATA_HOME");
    drop(temp_dir);
}

#[test]
#[serial]
fn test_simple_config_history() {
    let (manager, temp_dir) = create_isolated_config_manager();

    let client = MockClientBuilder::new("history-client").build();

    // Get initial history count
    let initial_history = manager
        .get_history(Some("history-client"), Some("tracked-server"))
        .unwrap();
    let initial_count = initial_history.len();

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

    // Debug: print all history entries
    println!("Initial history entries: {initial_count}");
    println!("Total history entries after test: {}", history.len());

    // We should have added exactly 3 entries
    assert_eq!(
        history.len(),
        initial_count + 3,
        "Expected to add exactly 3 history entries, but had {} initially and {} now",
        initial_count,
        history.len()
    );

    // Take the newest 3 entries
    let newest_entries: Vec<_> = history.iter().take(3).collect();

    // These should all be for our client/server
    for entry in &newest_entries {
        assert_eq!(entry.client_name, "history-client");
        assert_eq!(entry.server_name, "tracked-server");
    }

    // History should be sorted by timestamp (newest first)
    for i in 0..newest_entries.len() - 1 {
        assert!(newest_entries[i].timestamp >= newest_entries[i + 1].timestamp);
    }

    // Clean up
    std::env::remove_var("XDG_DATA_HOME");
    drop(temp_dir);
}

#[test]
#[serial]
fn test_simple_config_diff() {
    let (manager, temp_dir) = create_isolated_config_manager();

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

    // Clean up
    std::env::remove_var("XDG_DATA_HOME");
    drop(temp_dir);
}

#[test]
#[serial]
fn test_simple_latest_snapshot() {
    let (manager, temp_dir) = create_isolated_config_manager();

    let client = MockClientBuilder::new("latest-client").build();

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

    // Clean up
    std::env::remove_var("XDG_DATA_HOME");
    drop(temp_dir);
}
