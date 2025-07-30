use crate::client::{HomeDirectoryProvider, McpClient, RealHomeDirectoryProvider, ServerConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// VS Code MCP client implementation
/// Note: VS Code MCP support requires GitHub Copilot and is only available in Agent mode
pub struct VSCodeClient {
    name: String,
    home_provider: Box<dyn HomeDirectoryProvider>,
}

impl VSCodeClient {
    pub fn new() -> Self {
        Self {
            name: "VS Code".to_string(),
            home_provider: Box::new(RealHomeDirectoryProvider),
        }
    }

    #[cfg(test)]
    pub fn new_with_provider(home_provider: Box<dyn HomeDirectoryProvider>) -> Self {
        Self {
            name: "VS Code".to_string(),
            home_provider,
        }
    }
}

impl Default for VSCodeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VSCodeClient {
    /// Get home directory with fallback
    fn get_home_directory(&self) -> PathBuf {
        self.home_provider.home_dir().unwrap_or_else(|| {
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
        })
    }

    /// Get VS Code extension directories
    fn get_extension_directories(&self) -> Vec<PathBuf> {
        let home = self.get_home_directory();
        let mut dirs = vec![
            home.join(".vscode").join("extensions"),
            home.join(".vscode-server").join("extensions"),
        ];

        // Add system extension directory if available
        if let Some(system_dir) = self.get_system_extension_directory() {
            dirs.push(system_dir);
        }

        dirs
    }

    /// Get system VS Code extension directory
    fn get_system_extension_directory(&self) -> Option<PathBuf> {
        self.home_provider.home_dir().and_then(|_| {
            directories::BaseDirs::new()?
                .data_local_dir()
                .parent()
                .map(|p| p.join("vscode").join("extensions").to_path_buf())
        })
    }

    /// Check if a directory contains GitHub Copilot extension
    fn directory_has_copilot(&self, dir: &Path) -> bool {
        if !dir.exists() {
            return false;
        }

        let Ok(entries) = fs::read_dir(dir) else {
            return false;
        };

        entries.flatten().any(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name.starts_with("github.copilot"))
                .unwrap_or(false)
        })
    }

    /// Check if GitHub Copilot extension is installed
    fn check_copilot_installed(&self) -> bool {
        self.get_extension_directories()
            .iter()
            .any(|dir| self.directory_has_copilot(dir))
    }
}

impl McpClient for VSCodeClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        // VS Code uses ~/.vscode/mcp.json
        self.get_home_directory().join(".vscode").join("mcp.json")
    }

    fn is_installed(&self) -> bool {
        // Check if VS Code config directory exists
        let vscode_dir = self.get_home_directory().join(".vscode");
        vscode_dir.exists()
    }

    fn add_server(&self, name: &str, config: ServerConfig) -> Result<()> {
        // Check for GitHub Copilot requirement
        if !self.check_copilot_installed() {
            eprintln!("⚠️  Warning: VS Code MCP support requires GitHub Copilot extension");
            eprintln!("   Please install GitHub Copilot and use it in Agent mode");
        }

        let config_path = self.config_path();

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Read existing config or create new one
        let mut vscode_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str::<VSCodeConfig>(&content)?
        } else {
            VSCodeConfig::default()
        };

        // Convert to VS Code's format
        let vscode_server = VSCodeServer {
            type_: "stdio".to_string(),
            command: config.command,
            args: config.args,
            env: config.env,
        };

        // Add or update server
        vscode_config
            .servers
            .insert(name.to_string(), vscode_server);

        // Write back to file atomically
        let json = serde_json::to_string_pretty(&vscode_config)?;
        let temp_file =
            NamedTempFile::new_in(config_path.parent().unwrap_or_else(|| Path::new(".")))?;
        fs::write(temp_file.path(), &json).context("Failed to write config to temporary file")?;
        temp_file
            .persist(&config_path)
            .with_context(|| format!("Failed to persist config to {config_path:#?}"))?;

        println!("📝 Note: VS Code MCP servers are only available in GitHub Copilot Agent mode");

        Ok(())
    }

    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
        let config_path = self.config_path();

        if !config_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&config_path)?;
        let vscode_config: VSCodeConfig = serde_json::from_str(&content)?;

        // Convert from VS Code's format
        let mut servers = HashMap::new();
        for (name, vscode_server) in vscode_config.servers {
            let config = ServerConfig {
                command: vscode_server.command,
                args: vscode_server.args,
                env: vscode_server.env,
            };
            servers.insert(name, config);
        }

        Ok(servers)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct VSCodeConfig {
    servers: HashMap<String, VSCodeServer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VSCodeServer {
    #[serde(rename = "type")]
    type_: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockHomeDirectoryProvider;
    use tempfile::TempDir;

    #[test]
    fn test_vscode_client_name() {
        let client = VSCodeClient::new();
        assert_eq!(client.name(), "VS Code");
    }

    #[test]
    fn test_vscode_config_path() {
        let client = VSCodeClient::new();
        let path = client.config_path();
        assert!(path.ends_with(".vscode/mcp.json"));
    }

    #[test]
    fn test_vscode_is_installed() {
        let client = VSCodeClient::new();
        // This test will vary based on whether VS Code is actually installed
        let _ = client.is_installed();
    }

    #[test]
    fn test_vscode_add_server() {
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = VSCodeClient::new_with_provider(mock_provider);

        let config = ServerConfig {
            command: "python".to_string(),
            args: vec!["server.py".to_string()],
            env: HashMap::new(),
        };

        let result = client.add_server("test-server", config);
        assert!(result.is_ok());

        // Verify the config was written correctly
        let config_path = temp_dir.path().join(".vscode").join("mcp.json");
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-server"));
        assert!(content.contains("\"type\": \"stdio\""));
        assert!(content.contains("python"));
    }

    #[test]
    fn test_vscode_list_servers_with_data() {
        let temp_dir = TempDir::new().unwrap();
        let mock_provider = Box::new(MockHomeDirectoryProvider::new(
            temp_dir.path().to_path_buf(),
        ));
        let client = VSCodeClient::new_with_provider(mock_provider);

        // Add a server first
        let config = ServerConfig {
            command: "deno".to_string(),
            args: vec!["run".to_string(), "server.ts".to_string()],
            env: HashMap::new(),
        };

        client.add_server("deno-server", config).unwrap();

        // List servers
        let servers = client.list_servers().unwrap();
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("deno-server"));

        let server = &servers["deno-server"];
        assert_eq!(server.command, "deno");
        assert_eq!(server.args, vec!["run", "server.ts"]);
    }

    #[test]
    fn test_check_copilot_installed() {
        let client = VSCodeClient::new();
        // This will return false in test environment
        let _ = client.check_copilot_installed();
    }
}
