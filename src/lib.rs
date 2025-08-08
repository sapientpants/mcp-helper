//! # MCP Helper Library
//!
//! MCP Helper is a cross-platform tool that eliminates compatibility issues when working with
//! Model Context Protocol (MCP) servers. It acts as a universal launcher and configuration
//! manager for MCP servers, making them "just work" on Windows, macOS, and Linux.
//!
//! ## Core Features
//!
//! - **Universal Server Support**: NPM packages, Docker images, GitHub repositories, and binary releases
//! - **Multi-Client Integration**: Claude Desktop, VS Code, Cursor, Windsurf, and Claude Code
//! - **Cross-Platform Compatibility**: Handles platform-specific differences automatically
//! - **Security Validation**: Validates server sources and warns about potentially unsafe packages
//! - **Dependency Management**: Automatically checks and installs required dependencies
//! - **Interactive Configuration**: Guides users through server setup with helpful prompts
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use mcp_helper::install::InstallCommand;
//!
//! // Install an MCP server
//! let mut installer = InstallCommand::new(false); // verbose = false
//! installer.execute("@modelcontextprotocol/server-filesystem")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`client`]: MCP client implementations (Claude Desktop, VS Code, etc.)
//! - [`server`]: MCP server types (NPM, Docker, Binary, Python)
//! - [`deps`]: Dependency checking and installation instructions
//! - [`install`]: Main installation command logic
//! - [`security`]: Security validation for server sources
//! - [`error`]: Error types and handling
//! - [`runner`]: Core server execution logic
//! - [`config`]: Configuration management utilities
//! - [`logging`]: Structured logging support
//!
//! ## Platform Support
//!
//! MCP Helper supports:
//! - **Windows**: 10/11 (x64, ARM64)
//! - **macOS**: 10.15+ (Intel, Apple Silicon)  
//! - **Linux**: Ubuntu, Debian, CentOS, Fedora, Arch, Alpine (x64, ARM64)

pub mod cache;
pub mod client;
pub mod config;
pub mod core;
pub mod deps;
pub mod error;
pub mod install;
pub mod logging;
pub mod runner;
pub mod security;
pub mod server;
pub mod utils;

// Test utilities module (always available in development/test builds)
#[cfg(any(test, debug_assertions))]
pub mod test_utils;

// Re-export Platform enum so it can be used in tests
pub use runner::Platform;

// Re-export error types for external use
pub use error::{McpError, Result};
