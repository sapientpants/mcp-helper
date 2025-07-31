use crate::client::{
    get_home_with_fallback, HomeDirectoryProvider, McpClient, RealHomeDirectoryProvider,
    ServerConfig,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Windsurf (Codeium) MCP client implementation
pub struct WindsurfClient {
    name: String,
    home_provider: Box<dyn HomeDirectoryProvider>,
}

impl WindsurfClient {
    pub fn new() -> Self {
        Self {
            name: "Windsurf".to_string(),
            home_provider: Box::new(RealHomeDirectoryProvider),
        }
    }

    #[cfg(test)]
    pub fn new_with_provider(home_provider: Box<dyn HomeDirectoryProvider>) -> Self {
        Self {
            name: "Windsurf".to_string(),
            home_provider,
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
        get_home_with_fallback(&*self.home_provider)
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json")
    }

    fn is_installed(&self) -> bool {
        // Check if Windsurf/Codeium config directory exists
        let windsurf_dir = get_home_with_fallback(&*self.home_provider)
            .join(".codeium")
            .join("windsurf");
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
            crate::utils::json_validator::deserialize_json_safe::<WindsurfConfig>(&content)?
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

        // Write back to file atomically with secure permissions
        let json = serde_json::to_string_pretty(&windsurf_config)?;
        crate::utils::secure_file::write_json_secure(&config_path, &json)
            .with_context(|| format!("Failed to write config to {config_path:#?}"))?;

        Ok(())
    }

    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
        let config_path = self.config_path();

        if !config_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&config_path)?;
        let windsurf_config: WindsurfConfig =
            crate::utils::json_validator::deserialize_json_safe(&content)?;

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
    use super::*;
    use crate::client::MockHomeDirectoryProvider;
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
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = WindsurfClient::new_with_provider(mock_provider);

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
        let config_path = temp_dir
            .path()
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json");
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-server"));
        assert!(content.contains("mcpServers"));
        assert!(content.contains("npx"));
        assert!(content.contains("API_KEY"));
    }

    #[test]
    fn test_windsurf_list_servers_empty() {
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = WindsurfClient::new_with_provider(mock_provider);

        let servers = client.list_servers().unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn test_windsurf_config_format() {
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = WindsurfClient::new_with_provider(mock_provider);

        // Add server without env vars
        let config = ServerConfig {
            command: "python3".to_string(),
            args: vec!["-m".to_string(), "server".to_string()],
            env: HashMap::new(),
        };

        client.add_server("python-server", config).unwrap();

        // Check the JSON format
        let config_path = temp_dir
            .path()
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json");
        let content = fs::read_to_string(&config_path).unwrap();

        // Env should not be present when empty
        assert!(!content.contains("\"env\": {}"));
        assert!(!content.contains("\"serverUrl\""));
    }
}
