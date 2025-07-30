pub mod claude_code;
pub mod claude_desktop;
pub mod cursor;
pub mod vscode;
pub mod windsurf;

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

pub use claude_code::ClaudeCodeClient;
pub use claude_desktop::ClaudeDesktopClient;
pub use cursor::CursorClient;
pub use vscode::VSCodeClient;
pub use windsurf::WindsurfClient;

use std::env;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

pub trait McpClient: Send + Sync {
    fn name(&self) -> &str;

    fn config_path(&self) -> PathBuf;

    fn is_installed(&self) -> bool;

    fn add_server(&self, name: &str, config: ServerConfig) -> Result<()>;

    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>>;
}

pub struct ClientRegistry {
    pub clients: Vec<Box<dyn McpClient>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    pub fn register(&mut self, client: Box<dyn McpClient>) {
        self.clients.push(client);
    }

    pub fn detect_installed(&self) -> Vec<&dyn McpClient> {
        self.clients
            .iter()
            .filter(|client| client.is_installed())
            .map(|client| client.as_ref())
            .collect()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&dyn McpClient> {
        self.clients
            .iter()
            .find(|client| client.name().eq_ignore_ascii_case(name))
            .map(|client| client.as_ref())
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for providing home directory paths
/// This abstraction allows us to mock directory resolution in tests
pub trait HomeDirectoryProvider: Send + Sync {
    /// Get the user's home directory
    fn home_dir(&self) -> Option<PathBuf>;
}

/// Real implementation using directories crate
pub struct RealHomeDirectoryProvider;

impl HomeDirectoryProvider for RealHomeDirectoryProvider {
    fn home_dir(&self) -> Option<PathBuf> {
        directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf())
    }
}

/// Mock implementation for testing
#[cfg(test)]
pub struct MockHomeDirectoryProvider {
    home_path: PathBuf,
}

#[cfg(test)]
impl MockHomeDirectoryProvider {
    pub fn new(home_path: PathBuf) -> Self {
        Self { home_path }
    }
}

#[cfg(test)]
impl HomeDirectoryProvider for MockHomeDirectoryProvider {
    fn home_dir(&self) -> Option<PathBuf> {
        Some(self.home_path.clone())
    }
}

/// Get home directory with environment variable fallback
/// This provides a common fallback mechanism when the home directory provider returns None
pub fn get_home_with_fallback(provider: &dyn HomeDirectoryProvider) -> PathBuf {
    provider.home_dir().unwrap_or_else(|| {
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

pub fn detect_clients() -> Vec<Box<dyn McpClient>> {
    let mut registry = ClientRegistry::new();

    // Register all available clients
    registry.register(Box::new(ClaudeCodeClient::new()));
    registry.register(Box::new(ClaudeDesktopClient::new()));
    registry.register(Box::new(CursorClient::new()));
    registry.register(Box::new(VSCodeClient::new()));
    registry.register(Box::new(WindsurfClient::new()));

    registry.clients
}
