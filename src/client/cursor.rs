use crate::client::{McpClient, ServerConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Cursor MCP client implementation
pub struct CursorClient {
    name: String,
}

impl CursorClient {
    pub fn new() -> Self {
        Self {
            name: "Cursor".to_string(),
        }
    }
}

impl Default for CursorClient {
    fn default() -> Self {
        Self::new()
    }
}

impl McpClient for CursorClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        // Cursor uses two config locations:
        // 1. Global: ~/.cursor/mcp.json
        // 2. Project: .cursor/mcp.json (in project root)
        // For installation, we'll use the global config
        let home = directories::BaseDirs::new()
            .expect("Could not determine base directories")
            .home_dir()
            .to_path_buf();
        home.join(".cursor").join("mcp.json")
    }

    fn is_installed(&self) -> bool {
        // Check if Cursor config directory exists
        let cursor_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(".cursor"))
            .unwrap_or_default();

        cursor_dir.exists()
    }

    fn add_server(&self, name: &str, config: ServerConfig) -> Result<()> {
        let config_path = self.config_path();

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Read existing config or create new one
        let mut cursor_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str::<CursorConfig>(&content)?
        } else {
            CursorConfig::default()
        };

        // Convert to Cursor's format
        let cursor_server = CursorServer {
            type_: "stdio".to_string(),
            command: config.command,
            args: config.args,
            env: config.env,
        };

        // Add or update server
        cursor_config
            .servers
            .insert(name.to_string(), cursor_server);

        // Write back to file atomically
        let json = serde_json::to_string_pretty(&cursor_config)?;
        let temp_file =
            NamedTempFile::new_in(config_path.parent().unwrap_or_else(|| Path::new(".")))?;
        fs::write(temp_file.path(), &json).context("Failed to write config to temporary file")?;
        temp_file
            .persist(&config_path)
            .with_context(|| format!("Failed to persist config to {config_path:#?}"))?;

        Ok(())
    }

    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
        let config_path = self.config_path();

        if !config_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&config_path)?;
        let cursor_config: CursorConfig = serde_json::from_str(&content)?;

        // Convert from Cursor's format
        let mut servers = HashMap::new();
        for (name, cursor_server) in cursor_config.servers {
            let config = ServerConfig {
                command: cursor_server.command,
                args: cursor_server.args,
                env: cursor_server.env,
            };
            servers.insert(name, config);
        }

        Ok(servers)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct CursorConfig {
    servers: HashMap<String, CursorServer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CursorServer {
    #[serde(rename = "type")]
    type_: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    // NOTE: These tests modify HOME environment variable and should be run with --test-threads=1
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_cursor_client_name() {
        let client = CursorClient::new();
        assert_eq!(client.name(), "Cursor");
    }

    #[test]
    fn test_cursor_config_path() {
        let client = CursorClient::new();
        let path = client.config_path();
        assert!(path.ends_with(".cursor/mcp.json"));
    }

    #[test]
    fn test_cursor_is_installed() {
        let client = CursorClient::new();
        // This test will vary based on whether Cursor is actually installed
        let _ = client.is_installed();
    }

    #[test]
    fn test_cursor_add_server() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_path_buf();

        // Temporarily override HOME for this test
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

        let client = CursorClient::new();

        let config = ServerConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: HashMap::new(),
        };

        let result = client.add_server("test-server", config);
        assert!(result.is_ok());

        // Verify the config was written correctly
        let config_path = temp_home.join(".cursor").join("mcp.json");
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-server"));
        assert!(content.contains("\"type\": \"stdio\""));

        // Restore original HOME
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
    fn test_cursor_list_servers_empty() {
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

        let client = CursorClient::new();
        let servers = client.list_servers().unwrap();
        assert!(servers.is_empty());

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
}
