use crate::client::{McpClient, ServerConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

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

        serde_json::from_str(&content)
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

        // Write atomically using a temporary file
        let temp_file =
            NamedTempFile::new_in(self.config_path.parent().unwrap_or_else(|| Path::new(".")))?;

        fs::write(temp_file.path(), json).context("Failed to write config to temporary file")?;

        temp_file
            .persist(&self.config_path)
            .with_context(|| format!("Failed to persist config to {:#?}", self.config_path))?;

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
