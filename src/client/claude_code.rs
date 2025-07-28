use crate::client::{McpClient, ServerConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Claude Code MCP client implementation
pub struct ClaudeCodeClient {
    name: String,
}

impl ClaudeCodeClient {
    pub fn new() -> Self {
        Self {
            name: "Claude Code".to_string(),
        }
    }
}

impl Default for ClaudeCodeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl McpClient for ClaudeCodeClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        // Claude Code uses ~/.claude.json for user MCP servers
        let home = directories::BaseDirs::new()
            .expect("Could not determine base directories")
            .home_dir()
            .to_path_buf();
        home.join(".claude.json")
    }

    fn is_installed(&self) -> bool {
        // Check if Claude Code is installed by looking for the CLI tool
        // Claude Code can be invoked as 'claude' from the command line
        which::which("claude").is_ok()
    }

    fn add_server(&self, name: &str, config: ServerConfig) -> Result<()> {
        let config_path = self.config_path();

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Read existing config or create new one
        let mut claude_code_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str::<ClaudeCodeConfig>(&content)?
        } else {
            ClaudeCodeConfig::default()
        };

        // Initialize mcpServers if not present
        if claude_code_config.mcp_servers.is_none() {
            claude_code_config.mcp_servers = Some(HashMap::new());
        }

        // Convert to Claude Code's format
        let claude_code_server = ClaudeCodeServer {
            command: config.command,
            args: config.args,
            env: if config.env.is_empty() {
                None
            } else {
                Some(config.env)
            },
        };

        // Add or update server
        if let Some(ref mut servers) = claude_code_config.mcp_servers {
            servers.insert(name.to_string(), claude_code_server);
        }

        // Write back to file atomically
        let json = serde_json::to_string_pretty(&claude_code_config)?;
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
        let claude_code_config: ClaudeCodeConfig = serde_json::from_str(&content)?;

        // Convert from Claude Code's format
        let mut servers = HashMap::new();
        if let Some(mcp_servers) = claude_code_config.mcp_servers {
            for (name, claude_code_server) in mcp_servers {
                let config = ServerConfig {
                    command: claude_code_server.command,
                    args: claude_code_server.args,
                    env: claude_code_server.env.unwrap_or_default(),
                };
                servers.insert(name, config);
            }
        }

        Ok(servers)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ClaudeCodeConfig {
    #[serde(rename = "mcpServers", skip_serializing_if = "Option::is_none")]
    mcp_servers: Option<HashMap<String, ClaudeCodeServer>>,
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeCodeServer {
    command: String,
    args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    // NOTE: These tests modify HOME environment variable and should be run with --test-threads=1
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_claude_code_client_name() {
        let client = ClaudeCodeClient::new();
        assert_eq!(client.name(), "Claude Code");
    }

    #[test]
    fn test_claude_code_config_path() {
        let client = ClaudeCodeClient::new();
        let path = client.config_path();
        assert!(path.ends_with(".claude.json"));
    }

    #[test]
    fn test_claude_code_is_installed() {
        let client = ClaudeCodeClient::new();
        // This test will vary based on whether Claude Code is actually installed
        let _ = client.is_installed();
    }

    #[test]
    fn test_claude_code_add_server() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_path_buf();

        // Temporarily override HOME for this test
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", &temp_home);

        let client = ClaudeCodeClient::new();

        let config = ServerConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: HashMap::new(),
        };

        let result = client.add_server("test-server", config);
        assert!(result.is_ok());

        // Verify the config was written correctly
        let config_path = temp_home.join(".claude.json");
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-server"));
        assert!(content.contains("mcpServers"));
        assert!(content.contains("node"));

        // Restore original HOME
        match original_home {
            Some(home) => env::set_var("HOME", home),
            None => env::remove_var("HOME"),
        }
    }

    #[test]
    fn test_claude_code_list_servers_empty() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_path_buf();

        let original_home = env::var("HOME").ok();
        env::set_var("HOME", &temp_home);

        let client = ClaudeCodeClient::new();
        let servers = client.list_servers().unwrap();
        assert!(servers.is_empty());

        match original_home {
            Some(home) => env::set_var("HOME", home),
            None => env::remove_var("HOME"),
        }
    }

    #[test]
    fn test_claude_code_with_env_vars() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_path_buf();

        let original_home = env::var("HOME").ok();
        env::set_var("HOME", &temp_home);

        let client = ClaudeCodeClient::new();

        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), "test-key".to_string());

        let config = ServerConfig {
            command: "npx".to_string(),
            args: vec!["mcp-server".to_string()],
            env,
        };

        client.add_server("env-test", config).unwrap();

        // List servers and verify
        let servers = client.list_servers().unwrap();
        assert_eq!(servers.len(), 1);
        let server = &servers["env-test"];
        assert_eq!(server.env.get("API_KEY"), Some(&"test-key".to_string()));

        match original_home {
            Some(home) => env::set_var("HOME", home),
            None => env::remove_var("HOME"),
        }
    }

    #[test]
    fn test_claude_code_preserves_other_settings() {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_path_buf();

        let original_home = env::var("HOME").ok();
        env::set_var("HOME", &temp_home);

        // Create config with existing settings
        let config_path = temp_home.join(".claude.json");

        let existing_config = r#"{
            "theme": "dark",
            "fontSize": 14,
            "otherSetting": true
        }"#;
        fs::write(&config_path, existing_config).unwrap();

        let client = ClaudeCodeClient::new();

        let config = ServerConfig {
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
        };

        client.add_server("test-server", config).unwrap();

        // Verify other settings are preserved
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"theme\": \"dark\""));
        assert!(content.contains("\"fontSize\": 14"));
        assert!(content.contains("\"otherSetting\": true"));
        assert!(content.contains("mcpServers"));

        match original_home {
            Some(home) => env::set_var("HOME", home),
            None => env::remove_var("HOME"),
        }
    }
}
