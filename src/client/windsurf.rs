use crate::client::{McpClient, ServerConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Windsurf (Codeium) MCP client implementation
pub struct WindsurfClient {
    name: String,
}

impl WindsurfClient {
    pub fn new() -> Self {
        Self {
            name: "Windsurf".to_string(),
        }
    }
}

impl Default for WindsurfClient {
    fn default() -> Self {
        Self::new()
    }
}

impl McpClient for WindsurfClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        // Windsurf uses ~/.codeium/windsurf/mcp_config.json
        let home = if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.home_dir().to_path_buf()
        } else {
            // Fallback to environment variables if BaseDirs can't be determined
            #[cfg(windows)]
            {
                PathBuf::from(
                    env::var("USERPROFILE")
                        .unwrap_or_else(|_| env::var("HOME").unwrap_or_else(|_| ".".to_string())),
                )
            }
            #[cfg(not(windows))]
            {
                PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            }
        };
        home.join(".codeium")
            .join("windsurf")
            .join("mcp_config.json")
    }

    fn is_installed(&self) -> bool {
        // Check if Windsurf/Codeium config directory exists
        let home = if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.home_dir().to_path_buf()
        } else {
            // Fallback to environment variables if BaseDirs can't be determined
            #[cfg(windows)]
            {
                PathBuf::from(
                    env::var("USERPROFILE")
                        .unwrap_or_else(|_| env::var("HOME").unwrap_or_else(|_| ".".to_string())),
                )
            }
            #[cfg(not(windows))]
            {
                PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            }
        };

        let windsurf_dir = home.join(".codeium").join("windsurf");
        windsurf_dir.exists()
    }

    fn add_server(&self, name: &str, config: ServerConfig) -> Result<()> {
        let config_path = self.config_path();

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Read existing config or create new one
        let mut windsurf_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str::<WindsurfConfig>(&content)?
        } else {
            WindsurfConfig::default()
        };

        // Convert to Windsurf's format
        // Note: Windsurf uses "serverUrl" for remote servers and "command" for local
        let windsurf_server = WindsurfServer {
            command: Some(config.command),
            args: Some(config.args),
            env: if config.env.is_empty() {
                None
            } else {
                Some(config.env)
            },
            server_url: None, // For local servers
        };

        // Add or update server
        windsurf_config
            .mcp_servers
            .insert(name.to_string(), windsurf_server);

        // Write back to file atomically
        let json = serde_json::to_string_pretty(&windsurf_config)?;
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
        let windsurf_config: WindsurfConfig = serde_json::from_str(&content)?;

        // Convert from Windsurf's format
        let mut servers = HashMap::new();
        for (name, windsurf_server) in windsurf_config.mcp_servers {
            // Only include local servers (those with command)
            if let Some(command) = windsurf_server.command {
                let config = ServerConfig {
                    command,
                    args: windsurf_server.args.unwrap_or_default(),
                    env: windsurf_server.env.unwrap_or_default(),
                };
                servers.insert(name, config);
            }
        }

        Ok(servers)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct WindsurfConfig {
    #[serde(rename = "mcpServers")]
    mcp_servers: HashMap<String, WindsurfServer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WindsurfServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
    #[serde(rename = "serverUrl", skip_serializing_if = "Option::is_none")]
    server_url: Option<String>,
}

#[cfg(test)]
mod tests {
    // NOTE: These tests modify HOME environment variable and should be run with --test-threads=1
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_windsurf_client_name() {
        let client = WindsurfClient::new();
        assert_eq!(client.name(), "Windsurf");
    }

    #[test]
    fn test_windsurf_config_path() {
        let client = WindsurfClient::new();
        let path = client.config_path();
        assert!(path.ends_with(".codeium/windsurf/mcp_config.json"));
    }

    #[test]
    fn test_windsurf_is_installed() {
        let client = WindsurfClient::new();
        // This test will vary based on whether Windsurf is actually installed
        let _ = client.is_installed();
    }

    #[test]
    fn test_windsurf_add_server() {
        // Skip this test when running under coverage that has issues with env var manipulation
        if env::var("SKIP_ENV_TESTS").is_ok() {
            return;
        }

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

        let client = WindsurfClient::new();

        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), "test-key".to_string());

        let config = ServerConfig {
            command: "npx".to_string(),
            args: vec!["mcp-server".to_string()],
            env,
        };

        let result = client.add_server("test-server", config);
        assert!(result.is_ok());

        // Verify the config was written correctly
        let config_path = temp_home
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json");
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-server"));
        assert!(content.contains("mcpServers"));
        assert!(content.contains("npx"));
        assert!(content.contains("API_KEY"));

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
    fn test_windsurf_list_servers_empty() {
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

        let client = WindsurfClient::new();
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

    #[test]
    fn test_windsurf_config_format() {
        // Skip this test when running under coverage that has issues with env var manipulation
        if env::var("SKIP_ENV_TESTS").is_ok() {
            return;
        }

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

        let client = WindsurfClient::new();

        // Add server without env vars
        let config = ServerConfig {
            command: "python3".to_string(),
            args: vec!["-m".to_string(), "server".to_string()],
            env: HashMap::new(),
        };

        client.add_server("python-server", config).unwrap();

        // Check the JSON format
        let config_path = temp_home
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json");
        let content = fs::read_to_string(&config_path).unwrap();

        // Env should not be present when empty
        assert!(!content.contains("\"env\": {}"));
        assert!(!content.contains("\"serverUrl\""));

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
