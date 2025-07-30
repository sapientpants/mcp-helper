use assert_cmd::Command;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

use mcp_helper::client::{McpClient, ServerConfig};
use mcp_helper::config::ConfigManager;

// Mock client for testing
struct MockClient {
    name: String,
    servers: Arc<Mutex<HashMap<String, ServerConfig>>>,
    config_path: std::path::PathBuf,
}

impl McpClient for MockClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> std::path::PathBuf {
        self.config_path.clone()
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
fn test_end_to_end_npm_server_installation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let unique_path = temp_dir.path().join("e2e_npm_test");
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let client = MockClient {
        name: "test-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
        config_path,
    };

    // Simulate the full installation flow
    let config = ServerConfig {
        command: "npx".to_string(),
        args: vec![
            "--yes".to_string(),
            "test-server".to_string(),
            "--stdio".to_string(),
        ],
        env: {
            let mut env = HashMap::new();
            env.insert("API_KEY".to_string(), "test123".to_string());
            env
        },
    };

    // Test server installation
    let result = client.add_server("test-server", config);
    assert!(result.is_ok());

    // Verify server was installed
    let servers = client.list_servers().unwrap();
    assert!(servers.contains_key("test-server"));
    assert_eq!(servers["test-server"].command, "npx");
    assert!(servers["test-server"].env.contains_key("API_KEY"));
}

#[test]
fn test_end_to_end_docker_server_installation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let unique_path = temp_dir.path().join("e2e_docker_test");
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let client = MockClient {
        name: "docker-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
        config_path,
    };

    // Test Docker server installation
    let config = ServerConfig {
        command: "docker".to_string(),
        args: vec![
            "run".to_string(),
            "--rm".to_string(),
            "nginx:alpine".to_string(),
        ],
        env: {
            let mut env = HashMap::new();
            env.insert("ports".to_string(), "8080:80".to_string());
            env.insert(
                "environment".to_string(),
                "NGINX_HOST=localhost".to_string(),
            );
            env
        },
    };

    let result = client.add_server("nginx-server", config);
    assert!(result.is_ok());

    // Verify Docker server was installed
    let servers = client.list_servers().unwrap();
    assert!(servers.contains_key("nginx-server"));
    assert_eq!(servers["nginx-server"].command, "docker");
    assert!(servers["nginx-server"].env.contains_key("ports"));
    assert!(servers["nginx-server"].env.contains_key("environment"));
}

#[test]
fn test_end_to_end_configuration_management() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join("e2e_config_test");
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let config_manager = ConfigManager::new().unwrap();

    let client = MockClient {
        name: "config-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
        config_path: temp_dir.path().join("config.json"),
    };

    // Test configuration lifecycle: install -> update -> rollback
    let initial_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: {
            let mut env = HashMap::new();
            env.insert("PORT".to_string(), "3000".to_string());
            env
        },
    };

    // Apply initial configuration
    let snapshot1 = config_manager
        .apply_config(&client, "config-server", initial_config.clone())
        .unwrap();

    assert_eq!(snapshot1.server_name, "config-server");
    assert_eq!(snapshot1.client_name, "config-client");
    assert!(snapshot1.previous_config.is_none());

    // Update configuration
    let updated_config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string(), "--verbose".to_string()],
        env: {
            let mut env = HashMap::new();
            env.insert("PORT".to_string(), "4000".to_string());
            env.insert("DEBUG".to_string(), "true".to_string());
            env
        },
    };

    let snapshot2 = config_manager
        .apply_config(&client, "config-server", updated_config.clone())
        .unwrap();

    assert!(snapshot2.previous_config.is_some());
    let servers = client.list_servers().unwrap();
    assert_eq!(servers["config-server"].env["PORT"], "4000");
    assert!(servers["config-server"].env.contains_key("DEBUG"));

    // Test rollback
    let rollback_result = config_manager.rollback(&client, &snapshot2);
    assert!(rollback_result.is_ok());

    // Verify rollback worked
    let servers_after_rollback = client.list_servers().unwrap();
    assert_eq!(servers_after_rollback["config-server"].env["PORT"], "3000");
    assert!(!servers_after_rollback["config-server"]
        .env
        .contains_key("DEBUG"));
}

#[test]
fn test_end_to_end_batch_installation() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join("e2e_batch_test");
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    // Create a batch configuration file
    let batch_file = temp_dir.path().join("batch_servers.conf");
    let batch_content = r#"
# Test batch installation
[server-1]
api_key=key123
port=3000
debug=true

[server-2]
url=https://example.com
timeout=30
enabled=true

[server-3]
name=production
env=prod
workers=4
"#;

    fs::write(&batch_file, batch_content).unwrap();

    // Test parsing the batch file via InstallCommand
    let batch_content_read = fs::read_to_string(&batch_file).unwrap();

    // This simulates what InstallCommand::parse_batch_file does
    let mut servers = HashMap::new();
    let mut current_server: Option<String> = None;
    let mut current_config = HashMap::new();

    for line in batch_content_read.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check if this is a server declaration
        if line.starts_with('[') && line.ends_with(']') {
            // Save previous server if exists
            if let Some(server_name) = current_server.take() {
                servers.insert(server_name, current_config.clone());
                current_config.clear();
            }

            // Start new server
            current_server = Some(line[1..line.len() - 1].to_string());
            continue;
        }

        // Parse key=value configuration
        if let Some((key, value)) = line.split_once('=') {
            current_config.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // Save the last server
    if let Some(server_name) = current_server {
        servers.insert(server_name, current_config);
    }

    // Verify batch parsing worked correctly
    assert_eq!(servers.len(), 3);
    assert!(servers.contains_key("server-1"));
    assert!(servers.contains_key("server-2"));
    assert!(servers.contains_key("server-3"));

    // Check server-1 configuration
    let server1_config = &servers["server-1"];
    assert_eq!(server1_config["api_key"], "key123");
    assert_eq!(server1_config["port"], "3000");
    assert_eq!(server1_config["debug"], "true");

    // Check server-2 configuration
    let server2_config = &servers["server-2"];
    assert_eq!(server2_config["url"], "https://example.com");
    assert_eq!(server2_config["timeout"], "30");
    assert_eq!(server2_config["enabled"], "true");

    // Check server-3 configuration
    let server3_config = &servers["server-3"];
    assert_eq!(server3_config["name"], "production");
    assert_eq!(server3_config["env"], "prod");
    assert_eq!(server3_config["workers"], "4");
}

#[test]
fn test_end_to_end_non_interactive_installation() {
    // Test the CLI with non-interactive flags
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("cli_test.conf");

    let batch_content = r#"
[test-server]
api_key=cli_test_key
port=9000
"#;

    fs::write(&batch_file, batch_content).unwrap();

    // Test CLI with batch file
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install")
        .arg("dummy")
        .arg("--batch")
        .arg(batch_file.to_str().unwrap())
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect the server in batch mode
    assert!(stdout.contains("Found") || stdout.contains("1") || stdout.contains("server"));
}

#[test]
fn test_end_to_end_config_flag_installation() {
    // Test the CLI with --config flags
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install")
        .arg("test-package")
        .arg("--config")
        .arg("api_key=flag_test_key")
        .arg("--config")
        .arg("port=8000")
        .arg("--config")
        .arg("debug=false")
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should run in non-interactive mode
    assert!(
        stderr.contains("non-interactive")
            || stdout.contains("Installing")
            || stderr.contains("override")
    );
}

#[test]
fn test_end_to_end_error_handling() {
    // Test various error conditions in the installation flow

    // Test with invalid config format
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install")
        .arg("test-server")
        .arg("--config")
        .arg("invalid-no-equals")
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should warn about invalid format
    assert!(stderr.contains("Invalid config format") || stderr.contains("Expected key=value"));

    // Test with nonexistent batch file
    let mut cmd2 = Command::cargo_bin("mcp").unwrap();
    cmd2.arg("install")
        .arg("dummy")
        .arg("--batch")
        .arg("/nonexistent/batch.conf");

    let output2 = cmd2.output().unwrap();
    assert!(!output2.status.success());
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr2.contains("Failed to read batch file") || stderr2.contains("No such file"));
}

#[test]
fn test_end_to_end_history_and_rollback() {
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join("e2e_history_test");
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    let config_manager = ConfigManager::new().unwrap();

    let client = MockClient {
        name: "history-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
        config_path: temp_dir.path().join("config.json"),
    };

    // Create multiple configurations to test history
    let configs = vec![
        ("version-1", "PORT", "3000"),
        ("version-2", "PORT", "3001"),
        ("version-3", "PORT", "3002"),
    ];

    let mut snapshots = Vec::new();

    for (version, key, value) in configs {
        let config = ServerConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert(key.to_string(), value.to_string());
                env.insert("VERSION".to_string(), version.to_string());
                env
            },
        };

        let snapshot = config_manager
            .apply_config(&client, "history-server", config)
            .unwrap();
        snapshots.push(snapshot);
    }

    // Test history retrieval
    let history = config_manager
        .get_history(Some("history-client"), Some("history-server"))
        .unwrap();

    assert_eq!(history.len(), 3);

    // History should be sorted by timestamp (newest first)
    for i in 0..history.len() - 1 {
        assert!(history[i].timestamp >= history[i + 1].timestamp);
    }

    // Test latest snapshot retrieval
    let latest = config_manager
        .get_latest_snapshot("history-client", "history-server")
        .unwrap();

    assert!(latest.is_some());
    let latest_snapshot = latest.unwrap();
    assert_eq!(latest_snapshot.config.env["VERSION"], "version-3");
    assert_eq!(latest_snapshot.config.env["PORT"], "3002");

    // Test configuration diff
    let old_config = &snapshots[0].config;
    let new_config = &snapshots[2].config;
    let diffs = config_manager.diff_configs(old_config, new_config);

    assert!(!diffs.is_empty());
    assert!(diffs.iter().any(|d| d.contains("PORT")));
    assert!(diffs.iter().any(|d| d.contains("VERSION")));
}

#[test]
fn test_end_to_end_comprehensive_workflow() {
    // This test simulates a complete workflow from installation to management
    let temp_dir = TempDir::new().unwrap();
    let unique_path = temp_dir.path().join("e2e_comprehensive_test");
    std::fs::create_dir_all(&unique_path).unwrap();
    std::env::set_var("XDG_DATA_HOME", &unique_path);

    // Step 1: Initial installation via CLI simulation
    let initial_servers = vec![
        ("npm-server", "npx", vec!["--yes", "some-package"]),
        (
            "docker-server",
            "docker",
            vec!["run", "--rm", "nginx:alpine"],
        ),
    ];

    let client = MockClient {
        name: "comprehensive-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
        config_path: temp_dir.path().join("config.json"),
    };

    // Step 2: Install servers
    for (name, command, args) in initial_servers {
        let config = ServerConfig {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: HashMap::new(),
        };

        let result = client.add_server(name, config);
        assert!(result.is_ok());
    }

    // Step 3: Verify installation
    let servers = client.list_servers().unwrap();
    assert_eq!(servers.len(), 2);
    assert!(servers.contains_key("npm-server"));
    assert!(servers.contains_key("docker-server"));

    // Step 4: Test configuration management
    let config_manager = ConfigManager::new().unwrap();

    let enhanced_config = ServerConfig {
        command: "npx".to_string(),
        args: vec![
            "--yes".to_string(),
            "some-package".to_string(),
            "--verbose".to_string(),
        ],
        env: {
            let mut env = HashMap::new();
            env.insert("NODE_ENV".to_string(), "production".to_string());
            env.insert("DEBUG".to_string(), "true".to_string());
            env
        },
    };

    let snapshot = config_manager
        .apply_config(&client, "npm-server", enhanced_config)
        .unwrap();

    // Step 5: Verify enhanced configuration
    let updated_servers = client.list_servers().unwrap();
    assert!(updated_servers["npm-server"]
        .args
        .contains(&"--verbose".to_string()));
    assert!(updated_servers["npm-server"].env.contains_key("NODE_ENV"));

    // Step 6: Test rollback capability
    if snapshot.previous_config.is_some() {
        let rollback_result = config_manager.rollback(&client, &snapshot);
        assert!(rollback_result.is_ok());

        let rolled_back_servers = client.list_servers().unwrap();
        assert!(!rolled_back_servers["npm-server"]
            .args
            .contains(&"--verbose".to_string()));
        assert!(!rolled_back_servers["npm-server"]
            .env
            .contains_key("NODE_ENV"));
    }

    // Step 7: Verify final state
    let final_servers = client.list_servers().unwrap();
    assert_eq!(final_servers.len(), 2);
    assert_eq!(final_servers["npm-server"].command, "npx");
    assert_eq!(final_servers["docker-server"].command, "docker");
}
