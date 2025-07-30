use crate::client::{HomeDirectoryProvider, McpClient, RealHomeDirectoryProvider, ServerConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Claude Code MCP client implementation
pub struct ClaudeCodeClient {
    name: String,
    home_provider: Box<dyn HomeDirectoryProvider>,
}

impl ClaudeCodeClient {
    pub fn new() -> Self {
        Self {
            name: "Claude Code".to_string(),
            home_provider: Box::new(RealHomeDirectoryProvider),
        }
    }

    #[cfg(test)]
    pub fn new_with_provider(home_provider: Box<dyn HomeDirectoryProvider>) -> Self {
        Self {
            name: "Claude Code".to_string(),
            home_provider,
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
        let home = self.home_provider.home_dir().unwrap_or_else(|| {
            // Fallback to environment variables if home dir can't be determined
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
        });
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
    use super::*;
    use crate::client::MockHomeDirectoryProvider;
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
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = ClaudeCodeClient::new_with_provider(mock_provider);

        let config = ServerConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: HashMap::new(),
        };

        let result = client.add_server("test-server", config);
        assert!(result.is_ok(), "Failed to add server: {result:?}");

        // Verify the config was written correctly
        let config_path = temp_dir.path().join(".claude.json");
        assert!(
            config_path.exists(),
            "Config file should exist at {config_path:?}"
        );

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-server"));
        assert!(content.contains("mcpServers"));
        assert!(content.contains("node"));
    }

    #[test]
    fn test_claude_code_list_servers_empty() {
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = ClaudeCodeClient::new_with_provider(mock_provider);

        // Test listing servers when config doesn't exist
        let servers = client.list_servers().unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn test_claude_code_with_env_vars() {
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = ClaudeCodeClient::new_with_provider(mock_provider);

        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), "test-key".to_string());

        let config = ServerConfig {
            command: "npx".to_string(),
            args: vec!["mcp-server".to_string()],
            env,
        };

        client.add_server("env-test", config).unwrap();

        // Verify config file was created
        let config_path = temp_dir.path().join(".claude.json");
        assert!(
            config_path.exists(),
            "Config file should exist at {config_path:?}"
        );

        // List servers and verify
        let servers = client.list_servers().unwrap();
        let server_count = servers.len();
        assert_eq!(server_count, 1, "Expected 1 server, found {server_count}");
        let server_keys = servers.keys().collect::<Vec<_>>();
        assert!(
            servers.contains_key("env-test"),
            "Server 'env-test' not found in {server_keys:?}"
        );

        let server = &servers["env-test"];
        assert_eq!(server.env.get("API_KEY"), Some(&"test-key".to_string()));
    }

    #[test]
    fn test_claude_code_preserves_other_settings() {
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = ClaudeCodeClient::new_with_provider(mock_provider);

        // Create config with existing settings
        let config_path = temp_dir.path().join(".claude.json");

        let existing_config = r#"{
            "theme": "dark",
            "fontSize": 14,
            "otherSetting": true
        }"#;
        fs::write(&config_path, existing_config).unwrap();

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
    }

    #[test]
    fn test_claude_code_preserves_rich_user_data() {
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = ClaudeCodeClient::new_with_provider(mock_provider);

        // Create config with rich real-world data structure
        let config_path = temp_dir.path().join(".claude.json");

        let existing_config = r#"{
            "numStartups": 369,
            "installMethod": "unknown",
            "autoUpdates": true,
            "customApiKeyResponses": {
                "approved": ["3PYRUprgfmQ-oSMxHgAA"],
                "rejected": []
            },
            "tipsHistory": {
                "memory-command": 49,
                "theme-command": 352,
                "prompt-queue": 26,
                "todo-list": 359
            },
            "memoryUsageCount": 19,
            "promptQueueUseCount": 70,
            "autoUpdaterStatus": "enabled",
            "userID": "54e6bddd7992c80159c5d61f8cdfedca9e14c4af5ca0a391a5b94dbd59a82094",
            "hasCompletedOnboarding": true,
            "lastOnboardingVersion": "0.2.8",
            "projects": {
                "/Users/test/project1": {
                    "allowedTools": ["write", "read"],
                    "history": [
                        {
                            "display": "test command",
                            "pastedContents": {}
                        }
                    ],
                    "dontCrawlDirectory": false,
                    "mcpContextUris": [],
                    "mcpServers": {
                        "project-server": {
                            "command": "node",
                            "args": ["server.js"]
                        }
                    },
                    "enabledMcpjsonServers": [],
                    "disabledMcpjsonServers": [],
                    "hasTrustDialogAccepted": true
                }
            },
            "maxSubscriptionNoticeCount": 1,
            "hasAvailableMaxSubscription": false,
            "firstStartTime": "2025-05-13T04:28:26.892Z",
            "claudeMaxTier": "not_max",
            "hasSeenGAAnnounce": true,
            "mcpServers": {
                "existing-global": {
                    "command": "npx",
                    "args": ["-y", "some-mcp-server"]
                }
            }
        }"#;
        fs::write(&config_path, existing_config).unwrap();

        // Add a new MCP server
        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), "test-key".to_string());

        let config = ServerConfig {
            command: "node".to_string(),
            args: vec!["new-server.js".to_string()],
            env,
        };

        client.add_server("new-test-server", config).unwrap();

        // Read the config back and parse it
        let content = fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify all rich user data is preserved
        assert_eq!(parsed["numStartups"], 369);
        assert_eq!(parsed["installMethod"], "unknown");
        assert_eq!(parsed["autoUpdates"], true);
        assert_eq!(
            parsed["userID"],
            "54e6bddd7992c80159c5d61f8cdfedca9e14c4af5ca0a391a5b94dbd59a82094"
        );
        assert_eq!(parsed["hasCompletedOnboarding"], true);
        assert_eq!(parsed["lastOnboardingVersion"], "0.2.8");
        assert_eq!(parsed["memoryUsageCount"], 19);
        assert_eq!(parsed["promptQueueUseCount"], 70);
        assert_eq!(parsed["autoUpdaterStatus"], "enabled");
        assert_eq!(parsed["maxSubscriptionNoticeCount"], 1);
        assert_eq!(parsed["hasAvailableMaxSubscription"], false);
        assert_eq!(parsed["firstStartTime"], "2025-05-13T04:28:26.892Z");
        assert_eq!(parsed["claudeMaxTier"], "not_max");
        assert_eq!(parsed["hasSeenGAAnnounce"], true);

        // Verify customApiKeyResponses structure
        assert_eq!(
            parsed["customApiKeyResponses"]["approved"][0],
            "3PYRUprgfmQ-oSMxHgAA"
        );
        assert!(parsed["customApiKeyResponses"]["rejected"].is_array());

        // Verify tipsHistory
        assert_eq!(parsed["tipsHistory"]["memory-command"], 49);
        assert_eq!(parsed["tipsHistory"]["theme-command"], 352);
        assert_eq!(parsed["tipsHistory"]["prompt-queue"], 26);
        assert_eq!(parsed["tipsHistory"]["todo-list"], 359);

        // Verify projects structure is preserved including project-specific mcpServers
        let project = &parsed["projects"]["/Users/test/project1"];
        assert_eq!(project["allowedTools"][0], "write");
        assert_eq!(project["allowedTools"][1], "read");
        assert_eq!(project["history"][0]["display"], "test command");
        assert_eq!(project["dontCrawlDirectory"], false);
        assert_eq!(project["hasTrustDialogAccepted"], true);

        // Verify project-specific mcpServers are untouched
        assert_eq!(project["mcpServers"]["project-server"]["command"], "node");
        assert_eq!(
            project["mcpServers"]["project-server"]["args"][0],
            "server.js"
        );

        // Verify existing global server is preserved
        assert_eq!(parsed["mcpServers"]["existing-global"]["command"], "npx");
        assert_eq!(parsed["mcpServers"]["existing-global"]["args"][0], "-y");
        assert_eq!(
            parsed["mcpServers"]["existing-global"]["args"][1],
            "some-mcp-server"
        );

        // Verify new server was added
        assert_eq!(parsed["mcpServers"]["new-test-server"]["command"], "node");
        assert_eq!(
            parsed["mcpServers"]["new-test-server"]["args"][0],
            "new-server.js"
        );
        assert_eq!(
            parsed["mcpServers"]["new-test-server"]["env"]["API_KEY"],
            "test-key"
        );
    }
}
