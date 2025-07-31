//! MCP client implementations and management.
//!
//! This module provides support for multiple MCP clients including Claude Desktop,
//! VS Code, Cursor, Windsurf, and Claude Code. Each client has platform-specific
//! configuration paths and formats.
//!
//! # Supported Clients
//!
//! - **Claude Desktop**: The primary MCP client from Anthropic
//! - **VS Code**: Microsoft's editor with GitHub Copilot integration
//! - **Cursor**: AI-powered code editor
//! - **Windsurf**: Codeium-based MCP client
//! - **Claude Code**: Command-line interface
//!
//! # Usage
//!
//! ```rust,no_run
//! use mcp_helper::client::{detect_clients, McpClient, ServerConfig};
//! use std::collections::HashMap;
//!
//! // Detect all installed MCP clients
//! let clients = detect_clients();
//! for client in &clients {
//!     if client.is_installed() {
//!         println!("Found client: {}", client.name());
//!     }
//! }
//!
//! // Add a server to a client
//! let config = ServerConfig {
//!     command: "npx".to_string(),
//!     args: vec!["@modelcontextprotocol/server-filesystem".to_string()],
//!     env: HashMap::new(),
//! };
//! // client.add_server("filesystem", config)?;
//! ```

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

/// Configuration for an MCP server that can be added to a client.
///
/// This structure represents how an MCP server should be executed,
/// including the command, arguments, and environment variables.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    /// The command to execute (e.g., "npx", "python", "docker")
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
    /// Environment variables to set when running the server
    pub env: HashMap<String, String>,
}

/// Trait defining the interface for MCP clients.
///
/// Each MCP client (Claude Desktop, VS Code, etc.) implements this trait
/// to provide a unified way to manage server configurations across different clients.
pub trait McpClient: Send + Sync {
    /// Get the name of this client (e.g., "Claude Desktop", "VS Code").
    fn name(&self) -> &str;

    /// Get the path to this client's configuration file.
    fn config_path(&self) -> PathBuf;

    /// Check if this client is installed and available on the system.
    fn is_installed(&self) -> bool;

    /// Add a server configuration to this client.
    ///
    /// # Arguments
    /// * `name` - Unique name for the server within this client
    /// * `config` - Server configuration including command and arguments
    fn add_server(&self, name: &str, config: ServerConfig) -> Result<()>;

    /// List all servers currently configured for this client.
    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>>;
}

/// Registry for managing multiple MCP clients.
///
/// The ClientRegistry allows you to register multiple client implementations
/// and query them by name or find all installed clients.
pub struct ClientRegistry {
    /// Collection of registered MCP clients
    pub clients: Vec<Box<dyn McpClient>>,
}

impl ClientRegistry {
    /// Create a new empty client registry.
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    /// Register a new MCP client with the registry.
    ///
    /// # Arguments
    /// * `client` - Boxed client implementation to register
    pub fn register(&mut self, client: Box<dyn McpClient>) {
        self.clients.push(client);
    }

    /// Get all clients that are currently installed on the system.
    ///
    /// This method checks each registered client's `is_installed()` method
    /// and returns only those that are available.
    pub fn detect_installed(&self) -> Vec<&dyn McpClient> {
        self.clients
            .iter()
            .filter(|client| client.is_installed())
            .map(|client| client.as_ref())
            .collect()
    }

    /// Find a client by name (case-insensitive).
    ///
    /// # Arguments
    /// * `name` - Name of the client to find (e.g., "claude desktop", "vs code")
    ///
    /// # Returns
    /// The client if found, or None if no client matches the name
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

/// Detect and return all available MCP clients.
///
/// This function creates instances of all supported MCP client implementations
/// and returns them as a vector. Use this as a convenient way to get all
/// client types without manually creating each one.
///
/// # Returns
/// A vector containing instances of all supported MCP clients:
/// - Claude Code
/// - Claude Desktop  
/// - Cursor
/// - VS Code
/// - Windsurf
///
/// # Example
/// ```rust,no_run
/// use mcp_helper::client::detect_clients;
///
/// let clients = detect_clients();
/// for client in &clients {
///     if client.is_installed() {
///         println!("Found installed client: {}", client.name());
///     }
/// }
/// ```
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
