use crate::client::{McpClient, ServerConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ClaudeDesktopClient {
    config_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeConfig {
    #[serde(rename = "mcpServers", default)]
    mcp_servers: Map<String, Value>,

    #[serde(flatten)]
    other: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpServerConfig {
    command: String,
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
}

impl ClaudeDesktopClient {
    pub fn new() -> Self {
        Self {
            config_path: Self::get_config_path(),
        }
    }

    fn get_config_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                PathBuf::from(appdata)
                    .join("Claude")
                    .join("claude_desktop_config.json")
            } else {
                // Fallback using directories crate
                directories::BaseDirs::new()
                    .map(|dirs| {
                        dirs.config_dir()
                            .join("Claude")
                            .join("claude_desktop_config.json")
                    })
                    .unwrap_or_else(|| PathBuf::from("claude_desktop_config.json"))
            }
        }

        #[cfg(target_os = "macos")]
        {
            directories::BaseDirs::new()
                .map(|dirs| {
                    dirs.home_dir()
                        .join("Library")
                        .join("Application Support")
                        .join("Claude")
                        .join("claude_desktop_config.json")
                })
                .unwrap_or_else(|| PathBuf::from("claude_desktop_config.json"))
        }

        #[cfg(target_os = "linux")]
        {
            directories::BaseDirs::new()
                .map(|dirs| {
                    dirs.config_dir()
                        .join("Claude")
                        .join("claude_desktop_config.json")
                })
                .unwrap_or_else(|| PathBuf::from("claude_desktop_config.json"))
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            PathBuf::from("claude_desktop_config.json")
        }
    }

    fn read_config(&self) -> Result<ClaudeConfig> {
        if !self.config_path.exists() {
            // Return empty config if file doesn't exist
            return Ok(ClaudeConfig {
                mcp_servers: Map::new(),
                other: Map::new(),
            });
        }

        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read config from {:#?}", self.config_path))?;

        crate::utils::json_validator::deserialize_json_safe(&content)
            .with_context(|| format!("Failed to parse JSON from {:#?}", self.config_path))
    }

    fn write_config(&self, config: &ClaudeConfig) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {parent:?}"))?;
        }

        // Create backup if file exists
        if self.config_path.exists() {
            self.create_backup()?;
        }

        // Serialize with pretty printing
        let json =
            serde_json::to_string_pretty(config).context("Failed to serialize config to JSON")?;

        // Use secure file writing with proper permissions
        crate::utils::secure_file::write_json_secure(&self.config_path, &json)
            .with_context(|| format!("Failed to write config to {:#?}", self.config_path))?;

        Ok(())
    }

    fn create_backup(&self) -> Result<()> {
        let backup_path = self.config_path.with_extension("json.backup");
        fs::copy(&self.config_path, &backup_path)
            .with_context(|| format!("Failed to create backup at {backup_path:?}"))?;
        Ok(())
    }

    fn validate_config(config: &ServerConfig) -> Result<()> {
        if config.command.is_empty() {
            anyhow::bail!("Server command cannot be empty");
        }

        // Validate environment variables
        for key in config.env.keys() {
            if key.is_empty() {
                anyhow::bail!("Environment variable name cannot be empty");
            }
            if key.contains('=') {
                anyhow::bail!("Environment variable name cannot contain '='");
            }
        }

        Ok(())
    }
}

impl McpClient for ClaudeDesktopClient {
    fn name(&self) -> &str {
        "Claude Desktop"
    }

    fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    fn is_installed(&self) -> bool {
        // Check if Claude Desktop is installed by looking for the config directory
        if let Some(parent) = self.config_path.parent() {
            parent.exists()
        } else {
            false
        }
    }

    fn add_server(&self, name: &str, config: ServerConfig) -> Result<()> {
        // Validate the server config
        Self::validate_config(&config)?;

        // Read current config
        let mut claude_config = self.read_config()?;

        // Convert ServerConfig to JSON Value
        let server_value = serde_json::to_value(McpServerConfig {
            command: config.command,
            args: config.args,
            env: config.env,
        })?;

        // Add server to config
        claude_config
            .mcp_servers
            .insert(name.to_string(), server_value);

        // Write updated config
        self.write_config(&claude_config)?;

        Ok(())
    }

    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
        let config = self.read_config()?;
        let mut servers = HashMap::new();

        for (name, value) in config.mcp_servers {
            if let Ok(mcp_config) = serde_json::from_value::<McpServerConfig>(value) {
                servers.insert(
                    name,
                    ServerConfig {
                        command: mcp_config.command,
                        args: mcp_config.args,
                        env: mcp_config.env,
                    },
                );
            }
        }

        Ok(servers)
    }
}

impl Default for ClaudeDesktopClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_claude_desktop_client_new() {
        let client = ClaudeDesktopClient::new();
        assert!(!client.config_path.as_os_str().is_empty());
    }

    #[test]
    fn test_claude_desktop_client_default() {
        let client = ClaudeDesktopClient::default();
        assert!(!client.config_path.as_os_str().is_empty());
    }

    #[test]
    fn test_get_config_path_windows() {
        #[cfg(target_os = "windows")]
        {
            // Test with APPDATA set
            std::env::set_var("APPDATA", "C:\\Users\\Test\\AppData\\Roaming");
            let path = ClaudeDesktopClient::get_config_path();
            assert!(path.to_str().unwrap().contains("Claude"));
            assert!(path
                .to_str()
                .unwrap()
                .contains("claude_desktop_config.json"));
            std::env::remove_var("APPDATA");
        }
    }

    #[test]
    fn test_get_config_path_macos() {
        #[cfg(target_os = "macos")]
        {
            let path = ClaudeDesktopClient::get_config_path();
            assert!(path.to_str().unwrap().contains("Library"));
            assert!(path.to_str().unwrap().contains("Application Support"));
            assert!(path.to_str().unwrap().contains("Claude"));
        }
    }

    #[test]
    fn test_get_config_path_linux() {
        #[cfg(target_os = "linux")]
        {
            let path = ClaudeDesktopClient::get_config_path();
            assert!(path.to_str().unwrap().contains("Claude"));
            assert!(path
                .to_str()
                .unwrap()
                .contains("claude_desktop_config.json"));
        }
    }

    #[test]
    fn test_read_config_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        let config = client.read_config().unwrap();
        assert!(config.mcp_servers.is_empty());
        assert!(config.other.is_empty());
    }

    #[test]
    fn test_read_config_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let json_content = r#"{
            "mcpServers": {
                "test-server": {
                    "command": "node",
                    "args": ["server.js"],
                    "env": {"KEY": "value"}
                }
            },
            "otherField": "value"
        }"#;

        fs::write(&config_path, json_content).unwrap();

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        let config = client.read_config().unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        assert!(config.mcp_servers.contains_key("test-server"));
        assert_eq!(config.other.len(), 1);
        assert!(config.other.contains_key("otherField"));
    }

    #[test]
    fn test_write_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test").join("config.json");

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        let mut config = ClaudeConfig {
            mcp_servers: Map::new(),
            other: Map::new(),
        };

        config.mcp_servers.insert(
            "server1".to_string(),
            serde_json::json!({
                "command": "python",
                "args": ["-m", "server"],
                "env": {}
            }),
        );

        client.write_config(&config).unwrap();

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("mcpServers"));
        assert!(content.contains("server1"));
    }

    #[test]
    fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        fs::write(&config_path, "original content").unwrap();

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        client.create_backup().unwrap();

        let backup_path = config_path.with_extension("json.backup");
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "original content");
    }

    #[test]
    fn test_validate_config_empty_command() {
        let config = ServerConfig {
            command: String::new(),
            args: vec![],
            env: HashMap::new(),
        };

        let result = ClaudeDesktopClient::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_config_empty_env_key() {
        let mut env = HashMap::new();
        env.insert(String::new(), "value".to_string());

        let config = ServerConfig {
            command: "node".to_string(),
            args: vec![],
            env,
        };

        let result = ClaudeDesktopClient::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_config_env_key_with_equals() {
        let mut env = HashMap::new();
        env.insert("KEY=VALUE".to_string(), "value".to_string());

        let config = ServerConfig {
            command: "node".to_string(),
            args: vec![],
            env,
        };

        let result = ClaudeDesktopClient::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("="));
    }

    #[test]
    fn test_validate_config_valid() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/usr/bin".to_string());
        env.insert("NODE_ENV".to_string(), "production".to_string());

        let config = ServerConfig {
            command: "node".to_string(),
            args: vec![
                "server.js".to_string(),
                "--port".to_string(),
                "3000".to_string(),
            ],
            env,
        };

        assert!(ClaudeDesktopClient::validate_config(&config).is_ok());
    }

    #[test]
    fn test_name() {
        let client = ClaudeDesktopClient::new();
        assert_eq!(client.name(), "Claude Desktop");
    }

    #[test]
    fn test_config_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        assert_eq!(client.config_path(), config_path);
    }

    #[test]
    fn test_is_installed_with_parent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("Claude").join("config.json");

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        assert!(!client.is_installed());

        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        assert!(client.is_installed());
    }

    #[test]
    fn test_is_installed_no_parent() {
        let client = ClaudeDesktopClient {
            config_path: PathBuf::from("config.json"),
        };

        assert!(!client.is_installed());
    }

    #[test]
    fn test_add_server() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        let server_config = ServerConfig {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "mcp_server".to_string()],
            env: HashMap::from([("PYTHONPATH".to_string(), "/app".to_string())]),
        };

        client.add_server("test-server", server_config).unwrap();

        let servers = client.list_servers().unwrap();
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("test-server"));

        let added = &servers["test-server"];
        assert_eq!(added.command, "python");
        assert_eq!(added.args, vec!["-m", "mcp_server"]);
        assert_eq!(added.env.get("PYTHONPATH"), Some(&"/app".to_string()));
    }

    #[test]
    fn test_list_servers_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        let servers = client.list_servers().unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn test_list_servers_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let json_content = r#"{
            "mcpServers": {
                "server1": {
                    "command": "node",
                    "args": ["server1.js"],
                    "env": {}
                },
                "server2": {
                    "command": "python",
                    "args": ["server2.py"],
                    "env": {"KEY": "value"}
                }
            }
        }"#;

        fs::write(&config_path, json_content).unwrap();

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        let servers = client.list_servers().unwrap();
        assert_eq!(servers.len(), 2);
        assert!(servers.contains_key("server1"));
        assert!(servers.contains_key("server2"));

        assert_eq!(servers["server1"].command, "node");
        assert_eq!(servers["server2"].command, "python");
    }

    #[test]
    fn test_add_server_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Write initial config
        fs::write(&config_path, r#"{"mcpServers": {}}"#).unwrap();

        let client = ClaudeDesktopClient {
            config_path: config_path.clone(),
        };

        let server_config = ServerConfig {
            command: "node".to_string(),
            args: vec![],
            env: HashMap::new(),
        };

        client.add_server("new-server", server_config).unwrap();

        // Check backup was created
        let backup_path = config_path.with_extension("json.backup");
        assert!(backup_path.exists());

        // Check new server was added
        let servers = client.list_servers().unwrap();
        assert!(servers.contains_key("new-server"));
    }
}
