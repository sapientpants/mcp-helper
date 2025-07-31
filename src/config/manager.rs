use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::client::{McpClient, ServerConfig};
use crate::config::validator::{ConfigValidator, ValidationError};
use crate::server::McpServer;

/// Configuration snapshot for rollback support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub timestamp: DateTime<Utc>,
    pub client_name: String,
    pub server_name: String,
    pub config: ServerConfig,
    pub previous_config: Option<ServerConfig>,
    pub description: String,
}

/// Configuration history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHistory {
    pub snapshots: Vec<ConfigSnapshot>,
}

/// Configuration manager with validation and rollback support
pub struct ConfigManager {
    history_dir: PathBuf,
    max_history_entries: usize,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let history_dir = Self::get_history_dir()?;
        fs::create_dir_all(&history_dir)?;

        Ok(Self {
            history_dir,
            max_history_entries: 10,
        })
    }

    /// Validate a configuration before applying it
    pub fn validate_config(
        &self,
        server: &dyn McpServer,
        config: &HashMap<String, String>,
    ) -> Result<(), Vec<ValidationError>> {
        let metadata = server.metadata();
        ConfigValidator::validate_config(
            config,
            &metadata.required_config,
            &metadata.optional_config,
        )
    }

    /// Validate environment variables
    pub fn validate_env_vars(
        &self,
        env_vars: &HashMap<String, String>,
    ) -> Result<(), Vec<ValidationError>> {
        ConfigValidator::validate_env_vars(env_vars)
    }

    /// Test command availability
    pub fn test_command(&self, command: &str, args: &[String]) -> Result<()> {
        ConfigValidator::test_command_availability(command, args)
    }

    /// Apply configuration with automatic backup
    pub fn apply_config(
        &self,
        client: &dyn McpClient,
        server_name: &str,
        new_config: ServerConfig,
    ) -> Result<ConfigSnapshot> {
        // Get current configuration
        let current_servers = client.list_servers()?;
        let previous_config = current_servers.get(server_name).cloned();

        // Create snapshot before applying
        let snapshot = ConfigSnapshot {
            timestamp: Utc::now(),
            client_name: client.name().to_string(),
            server_name: server_name.to_string(),
            config: new_config.clone(),
            previous_config,
            description: format!("Configuration update for {server_name}"),
        };

        // Save snapshot to history
        self.save_snapshot(&snapshot)?;

        // Apply the new configuration
        client.add_server(server_name, new_config)?;

        Ok(snapshot)
    }

    /// Rollback to a previous configuration
    pub fn rollback(&self, client: &dyn McpClient, snapshot: &ConfigSnapshot) -> Result<()> {
        if let Some(ref previous_config) = snapshot.previous_config {
            // Restore the previous configuration
            client.add_server(&snapshot.server_name, previous_config.clone())?;

            // Create a rollback snapshot
            let rollback_snapshot = ConfigSnapshot {
                timestamp: Utc::now(),
                client_name: snapshot.client_name.clone(),
                server_name: snapshot.server_name.clone(),
                config: previous_config.clone(),
                previous_config: Some(snapshot.config.clone()),
                description: format!(
                    "Rollback from {} to previous configuration",
                    snapshot.timestamp.format("%Y-%m-%d %H:%M:%S")
                ),
            };

            self.save_snapshot(&rollback_snapshot)?;
        } else {
            // If there was no previous config, remove the server
            anyhow::bail!(
                "Cannot rollback: no previous configuration found for {}",
                snapshot.server_name
            );
        }

        Ok(())
    }

    /// Get configuration history for a specific client and server
    pub fn get_history(
        &self,
        client_name: Option<&str>,
        server_name: Option<&str>,
    ) -> Result<Vec<ConfigSnapshot>> {
        let history = self.load_history()?;
        let mut snapshots = history.snapshots;

        // Filter by client name if provided
        if let Some(client) = client_name {
            snapshots.retain(|s| s.client_name == client);
        }

        // Filter by server name if provided
        if let Some(server) = server_name {
            snapshots.retain(|s| s.server_name == server);
        }

        // Sort by timestamp (newest first)
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(snapshots)
    }

    /// Get the latest snapshot for a specific server
    pub fn get_latest_snapshot(
        &self,
        client_name: &str,
        server_name: &str,
    ) -> Result<Option<ConfigSnapshot>> {
        let history = self.get_history(Some(client_name), Some(server_name))?;
        Ok(history.into_iter().next())
    }

    /// Compare two configurations and return differences
    pub fn diff_configs(
        &self,
        old_config: &ServerConfig,
        new_config: &ServerConfig,
    ) -> Vec<String> {
        let mut differences = Vec::new();

        // Compare command
        if old_config.command != new_config.command {
            differences.push(format!(
                "Command: {} → {}",
                old_config.command, new_config.command
            ));
        }

        // Compare args
        if old_config.args != new_config.args {
            differences.push(format!(
                "Arguments: {:?} → {:?}",
                old_config.args, new_config.args
            ));
        }

        // Compare environment variables
        let old_keys: std::collections::HashSet<_> = old_config.env.keys().collect();
        let new_keys: std::collections::HashSet<_> = new_config.env.keys().collect();

        // Added env vars
        for key in new_keys.difference(&old_keys) {
            if let Some(value) = new_config.env.get(*key) {
                differences.push(format!("Added env var: {key}={value}"));
            }
        }

        // Removed env vars
        for key in old_keys.difference(&new_keys) {
            differences.push(format!("Removed env var: {key}"));
        }

        // Modified env vars
        for key in old_keys.intersection(&new_keys) {
            let old_val = old_config.env.get(*key);
            let new_val = new_config.env.get(*key);
            if old_val != new_val {
                differences.push(format!("Modified env var {key}: {old_val:?} → {new_val:?}"));
            }
        }

        differences
    }

    /// Clean up old history entries
    pub fn cleanup_history(&self) -> Result<()> {
        let mut history = self.load_history()?;

        if history.snapshots.len() > self.max_history_entries {
            // Sort by timestamp (oldest first)
            history
                .snapshots
                .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            // Keep only the most recent entries
            let to_remove = history.snapshots.len() - self.max_history_entries;
            history.snapshots.drain(..to_remove);
        }

        self.save_history(&history)?;
        Ok(())
    }

    fn get_history_dir() -> Result<PathBuf> {
        let base_dir = directories::ProjectDirs::from("", "", "mcp-helper")
            .context("Failed to get project directories")?;
        Ok(base_dir.data_dir().join("config-history"))
    }

    fn get_history_file(&self) -> PathBuf {
        self.history_dir.join("history.json")
    }

    fn load_history(&self) -> Result<ConfigHistory> {
        let history_file = self.get_history_file();

        if history_file.exists() {
            let contents =
                fs::read_to_string(&history_file).context("Failed to read history file")?;
            serde_json::from_str(&contents).context("Failed to parse history file")
        } else {
            Ok(ConfigHistory {
                snapshots: Vec::new(),
            })
        }
    }

    fn save_history(&self, history: &ConfigHistory) -> Result<()> {
        let history_file = self.get_history_file();
        let contents =
            serde_json::to_string_pretty(history).context("Failed to serialize history")?;

        crate::utils::secure_file::write_json_secure(&history_file, &contents)
            .context("Failed to write history file")?;

        Ok(())
    }

    fn save_snapshot(&self, snapshot: &ConfigSnapshot) -> Result<()> {
        let mut history = self.load_history()?;
        history.snapshots.push(snapshot.clone());
        self.save_history(&history)?;
        self.cleanup_history()?;
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create config manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::McpClient;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

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

        fn add_server(&self, name: &str, config: ServerConfig) -> Result<()> {
            self.servers
                .lock()
                .unwrap()
                .insert(name.to_string(), config);
            Ok(())
        }

        fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
            Ok(self.servers.lock().unwrap().clone())
        }
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_apply_and_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let unique_path = temp_dir.path().join(format!(
            "apply_rollback_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&unique_path).unwrap();
        std::env::set_var("XDG_DATA_HOME", &unique_path);

        let manager = ConfigManager::new().unwrap();

        // Clear any existing history
        let empty_history = ConfigHistory {
            snapshots: Vec::new(),
        };
        manager.save_history(&empty_history).unwrap();

        let client = MockClient {
            name: "test-client".to_string(),
            servers: Arc::new(Mutex::new(HashMap::new())),
        };

        // Apply initial config
        let config1 = ServerConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: HashMap::new(),
        };

        let _snapshot1 = manager
            .apply_config(&client, "test-server", config1.clone())
            .unwrap();

        // Apply updated config
        let mut config2 = config1.clone();
        config2.args.push("--port=3000".to_string());

        let snapshot2 = manager
            .apply_config(&client, "test-server", config2.clone())
            .unwrap();

        // Verify current config
        let servers = client.list_servers().unwrap();
        assert_eq!(servers["test-server"].args.len(), 2);

        // Rollback to previous config
        manager.rollback(&client, &snapshot2).unwrap();

        // Verify rollback worked
        let servers = client.list_servers().unwrap();
        assert_eq!(servers["test-server"].args.len(), 1);
    }

    #[test]
    fn test_config_diff() {
        let manager = ConfigManager::new().unwrap();

        let config1 = ServerConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: HashMap::from([("PORT".to_string(), "3000".to_string())]),
        };

        let mut config2 = config1.clone();
        config2.command = "deno".to_string();
        config2.args.push("--allow-net".to_string());
        config2.env.insert("DEBUG".to_string(), "true".to_string());
        config2.env.remove("PORT");

        let diffs = manager.diff_configs(&config1, &config2);

        assert!(diffs.iter().any(|d| d.contains("Command: node → deno")));
        assert!(diffs.iter().any(|d| d.contains("Arguments:")));
        assert!(diffs
            .iter()
            .any(|d| d.contains("Added env var: DEBUG=true")));
        assert!(diffs.iter().any(|d| d.contains("Removed env var: PORT")));
    }

    #[test]
    fn test_history_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let unique_path = temp_dir.path().join(format!(
            "history_filtering_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&unique_path).unwrap();
        std::env::set_var("XDG_DATA_HOME", &unique_path);

        let manager = ConfigManager::new().unwrap();

        // Clear any existing history
        let empty_history = ConfigHistory {
            snapshots: Vec::new(),
        };
        manager.save_history(&empty_history).unwrap();

        // Create some test snapshots
        let snapshot1 = ConfigSnapshot {
            timestamp: Utc::now(),
            client_name: "client1".to_string(),
            server_name: "server1".to_string(),
            config: ServerConfig {
                command: "test".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
            previous_config: None,
            description: "Test 1".to_string(),
        };

        let snapshot2 = ConfigSnapshot {
            timestamp: Utc::now(),
            client_name: "client2".to_string(),
            server_name: "server1".to_string(),
            config: ServerConfig {
                command: "test".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
            previous_config: None,
            description: "Test 2".to_string(),
        };

        manager.save_snapshot(&snapshot1).unwrap();
        manager.save_snapshot(&snapshot2).unwrap();

        // Test filtering by client
        let history = manager.get_history(Some("client1"), None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].client_name, "client1");

        // Test filtering by server
        let history = manager.get_history(None, Some("server1")).unwrap();
        assert_eq!(history.len(), 2);
    }
}
