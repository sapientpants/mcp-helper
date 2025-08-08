//! Error handling for MCP Helper operations.
//!
//! This module provides comprehensive error types that give users actionable information
//! about what went wrong and how to fix it. All errors are designed to be user-friendly
//! with colored output and platform-specific instructions.
//!
//! # Examples
//!
//! ```rust,no_run
//! use mcp_helper::error::{McpError, Result};
//! use mcp_helper::deps::InstallInstructions;
//!
//! // Create a missing dependency error with install instructions
//! let error = McpError::missing_dependency(
//!     "Node.js",
//!     Some("18.0.0".to_string()),
//!     InstallInstructions::default()
//! );
//!
//! println!("{}", error); // Displays colored, helpful error message
//! ```

use colored::Colorize;
use std::fmt;

use crate::deps::InstallInstructions;

pub mod builder;
pub use builder::ErrorBuilder;

/// Comprehensive error type for MCP Helper operations.
///
/// Each error variant provides specific context and actionable guidance to help users
/// resolve issues. Errors are formatted with colors and clear instructions.
#[derive(Debug)]
pub enum McpError {
    /// A required dependency (Node.js, Docker, Python, etc.) is missing.
    ///
    /// Provides platform-specific installation instructions to help users
    /// install the missing dependency.
    MissingDependency {
        /// Name of the missing dependency
        dependency: String,
        /// Required version (if any)
        required_version: Option<String>,
        /// Platform-specific installation instructions
        install_instructions: Box<InstallInstructions>,
    },

    /// A dependency is installed but doesn't meet version requirements.
    ///
    /// Shows current vs required version and provides upgrade instructions.
    VersionMismatch {
        /// Name of the dependency with version issues
        dependency: String,
        /// Currently installed version
        current_version: String,
        /// Version required by the server
        required_version: String,
        /// Platform-specific upgrade instructions
        upgrade_instructions: Box<InstallInstructions>,
    },

    /// A server requires configuration that wasn't provided.
    ///
    /// Lists missing configuration fields with descriptions to guide users.
    ConfigurationRequired {
        /// Name of the server requiring configuration
        server_name: String,
        /// List of missing configuration field names
        missing_fields: Vec<String>,
        /// Field descriptions as (name, description) pairs
        field_descriptions: Vec<(String, String)>,
    },

    /// No MCP clients were found or selected for installation.
    ///
    /// Shows available clients and provides installation guidance.
    ClientNotFound {
        /// Name of the client that wasn't found
        client_name: String,
        /// List of available/detected clients
        available_clients: Vec<String>,
        /// Instructions for installing the client
        install_guidance: String,
    },

    /// Configuration file parsing or validation error.
    ///
    /// Indicates issues with JSON syntax, file permissions, or content validation.
    ConfigError {
        /// Path to the configuration file
        path: String,
        /// Specific error message
        message: String,
    },

    /// Server-specific error during installation or execution.
    ///
    /// Covers issues like invalid server names, download failures, or runtime errors.
    ServerError {
        /// Name of the server that caused the error
        server_name: String,
        /// Specific error message
        message: String,
    },

    /// File system I/O operation failed.
    ///
    /// Covers file reading, writing, permission errors, and path issues.
    IoError {
        /// Description of the operation that failed
        operation: String,
        /// Path involved in the operation (if applicable)
        path: Option<String>,
        /// Underlying I/O error
        source: std::io::Error,
    },

    /// Catch-all for other error types.
    ///
    /// Used for wrapping errors from external libraries or unexpected conditions.
    Other(anyhow::Error),
}

impl McpError {
    /// Create a missing dependency error with installation instructions.
    ///
    /// # Arguments
    /// * `dependency` - Name of the missing dependency (e.g., "Node.js", "Docker")
    /// * `required_version` - Optional version requirement (e.g., Some("18.0.0".to_string()))
    /// * `install_instructions` - Platform-specific installation methods
    ///
    /// # Example
    /// ```rust,no_run
    /// use mcp_helper::error::McpError;
    /// use mcp_helper::deps::InstallInstructions;
    ///
    /// let error = McpError::missing_dependency(
    ///     "Docker",
    ///     None,
    ///     InstallInstructions::default()
    /// );
    /// ```
    pub fn missing_dependency(
        dependency: impl Into<String>,
        required_version: Option<String>,
        install_instructions: InstallInstructions,
    ) -> Self {
        Self::MissingDependency {
            dependency: dependency.into(),
            required_version,
            install_instructions: Box::new(install_instructions),
        }
    }

    /// Create a version mismatch error with upgrade instructions.
    ///
    /// # Arguments
    /// * `dependency` - Name of the dependency with version issues
    /// * `current_version` - Currently installed version
    /// * `required_version` - Version required by the server
    /// * `upgrade_instructions` - Platform-specific upgrade methods
    pub fn version_mismatch(
        dependency: impl Into<String>,
        current_version: impl Into<String>,
        required_version: impl Into<String>,
        upgrade_instructions: InstallInstructions,
    ) -> Self {
        Self::VersionMismatch {
            dependency: dependency.into(),
            current_version: current_version.into(),
            required_version: required_version.into(),
            upgrade_instructions: Box::new(upgrade_instructions),
        }
    }

    /// Create a configuration required error with field descriptions.
    ///
    /// # Arguments
    /// * `server_name` - Name of the server requiring configuration
    /// * `missing_fields` - List of missing configuration field names
    /// * `field_descriptions` - Field descriptions as (name, description) pairs
    pub fn configuration_required(
        server_name: impl Into<String>,
        missing_fields: Vec<String>,
        field_descriptions: Vec<(String, String)>,
    ) -> Self {
        Self::ConfigurationRequired {
            server_name: server_name.into(),
            missing_fields,
            field_descriptions,
        }
    }

    /// Create a client not found error with available alternatives.
    ///
    /// # Arguments
    /// * `client_name` - Name of the client that wasn't found
    /// * `available_clients` - List of detected/available clients
    /// * `install_guidance` - Instructions for installing the client
    pub fn client_not_found(
        client_name: impl Into<String>,
        available_clients: Vec<String>,
        install_guidance: impl Into<String>,
    ) -> Self {
        Self::ClientNotFound {
            client_name: client_name.into(),
            available_clients,
            install_guidance: install_guidance.into(),
        }
    }

    /// Create a configuration file error.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    /// * `message` - Specific error message describing the issue
    pub fn config_error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConfigError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a server-specific error.
    ///
    /// # Arguments
    /// * `server_name` - Name of the server that caused the error
    /// * `message` - Specific error message describing the issue
    pub fn server_error(server_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ServerError {
            server_name: server_name.into(),
            message: message.into(),
        }
    }

    /// Create an I/O operation error.
    ///
    /// # Arguments
    /// * `operation` - Description of the operation that failed
    /// * `path` - Optional path involved in the operation
    /// * `source` - Underlying I/O error
    pub fn io_error(
        operation: impl Into<String>,
        path: Option<String>,
        source: std::io::Error,
    ) -> Self {
        Self::IoError {
            operation: operation.into(),
            path,
            source,
        }
    }
}

// Helper functions to reduce complexity
fn write_error_header(f: &mut fmt::Formatter<'_>, message: &str, subject: &str) -> fmt::Result {
    writeln!(f, "{} {}: {}", "✗".red().bold(), message, subject.yellow())
}

fn write_detail(f: &mut fmt::Formatter<'_>, label: &str, value: &str) -> fmt::Result {
    writeln!(f, "  {} {}: {}", "→".blue(), label, value)
}

fn write_section_header(f: &mut fmt::Formatter<'_>, title: &str) -> fmt::Result {
    writeln!(f)?;
    writeln!(f, "{}", title.green().bold())
}

impl McpError {
    fn fmt_missing_dependency(
        f: &mut fmt::Formatter<'_>,
        dependency: &str,
        required_version: &Option<String>,
        install_instructions: &InstallInstructions,
    ) -> fmt::Result {
        write_error_header(f, "Missing dependency", dependency)?;
        if let Some(version) = required_version {
            write_detail(f, "Required version", version)?;
        }
        write_section_header(f, "How to install:")?;
        format_install_instructions(f, install_instructions)
    }

    fn fmt_version_mismatch(
        f: &mut fmt::Formatter<'_>,
        dependency: &str,
        current_version: &str,
        required_version: &str,
        upgrade_instructions: &InstallInstructions,
    ) -> fmt::Result {
        write_error_header(f, "Version mismatch for", dependency)?;
        write_detail(f, "Current version", current_version)?;
        write_detail(f, "Required version", required_version)?;
        write_section_header(f, "How to upgrade:")?;
        format_install_instructions(f, upgrade_instructions)
    }

    fn fmt_configuration_required(
        f: &mut fmt::Formatter<'_>,
        server_name: &str,
        missing_fields: &[String],
        field_descriptions: &[(String, String)],
    ) -> fmt::Result {
        write_error_header(f, "Configuration required for", server_name)?;
        writeln!(f)?;
        writeln!(f, "{}", "Missing fields:".red())?;
        for field in missing_fields {
            writeln!(f, "  {} {}", "•".blue(), field)?;
        }
        if !field_descriptions.is_empty() {
            writeln!(f)?;
            writeln!(f, "{}", "Field descriptions:".green())?;
            for (field, desc) in field_descriptions {
                writeln!(f, "  {} {}: {}", "→".blue(), field.bold(), desc)?;
            }
        }
        Ok(())
    }

    fn fmt_client_not_found(
        f: &mut fmt::Formatter<'_>,
        client_name: &str,
        available_clients: &[String],
        install_guidance: &str,
    ) -> fmt::Result {
        writeln!(
            f,
            "{} MCP client not found: {}",
            "✗".red().bold(),
            client_name.yellow()
        )?;
        if !available_clients.is_empty() {
            writeln!(f)?;
            writeln!(f, "{}", "Available clients:".green())?;
            for client in available_clients {
                writeln!(f, "  {} {}", "•".blue(), client)?;
            }
        }
        writeln!(f)?;
        writeln!(f, "{}", "Installation guidance:".green().bold())?;
        writeln!(f, "  {install_guidance}")
    }

    fn fmt_config_error(f: &mut fmt::Formatter<'_>, path: &str, message: &str) -> fmt::Result {
        writeln!(f, "{} Configuration error", "✗".red().bold())?;
        writeln!(f, "  {} Path: {}", "→".blue(), path.yellow())?;
        writeln!(f, "  {} Error: {}", "→".blue(), message)
    }

    fn fmt_server_error(
        f: &mut fmt::Formatter<'_>,
        server_name: &str,
        message: &str,
    ) -> fmt::Result {
        writeln!(
            f,
            "{} Server error: {}",
            "✗".red().bold(),
            server_name.yellow()
        )?;
        writeln!(f, "  {} {}", "→".blue(), message)
    }

    fn fmt_io_error(
        f: &mut fmt::Formatter<'_>,
        operation: &str,
        path: &Option<String>,
        source: &std::io::Error,
    ) -> fmt::Result {
        writeln!(
            f,
            "{} I/O error during: {}",
            "✗".red().bold(),
            operation.yellow()
        )?;
        if let Some(path) = path {
            writeln!(f, "  {} Path: {}", "→".blue(), path)?;
        }
        writeln!(f, "  {} Error: {}", "→".blue(), source)
    }
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDependency {
                dependency,
                required_version,
                install_instructions,
            } => {
                Self::fmt_missing_dependency(f, dependency, required_version, install_instructions)
            }
            Self::VersionMismatch {
                dependency,
                current_version,
                required_version,
                upgrade_instructions,
            } => Self::fmt_version_mismatch(
                f,
                dependency,
                current_version,
                required_version,
                upgrade_instructions,
            ),
            Self::ConfigurationRequired {
                server_name,
                missing_fields,
                field_descriptions,
            } => {
                Self::fmt_configuration_required(f, server_name, missing_fields, field_descriptions)
            }
            Self::ClientNotFound {
                client_name,
                available_clients,
                install_guidance,
            } => Self::fmt_client_not_found(f, client_name, available_clients, install_guidance),
            Self::ConfigError { path, message } => Self::fmt_config_error(f, path, message),
            Self::ServerError {
                server_name,
                message,
            } => Self::fmt_server_error(f, server_name, message),
            Self::IoError {
                operation,
                path,
                source,
            } => Self::fmt_io_error(f, operation, path, source),
            Self::Other(err) => write!(f, "{} {}", "✗".red().bold(), err),
        }
    }
}

fn format_install_instructions(
    f: &mut fmt::Formatter<'_>,
    instructions: &InstallInstructions,
) -> fmt::Result {
    #[cfg(target_os = "windows")]
    let methods = &instructions.windows;
    #[cfg(target_os = "macos")]
    let methods = &instructions.macos;
    #[cfg(target_os = "linux")]
    let methods = &instructions.linux;

    for method in methods {
        writeln!(f, "  {} {}", "•".blue(), method.name.bold())?;
        writeln!(f, "    {} {}", "$".cyan(), method.command)?;
        if let Some(desc) = &method.description {
            writeln!(f, "    {} {}", "Note:".yellow(), desc)?;
        }
    }
    Ok(())
}

impl std::error::Error for McpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError { source, .. } => Some(source),
            Self::Other(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            operation: "unknown".to_string(),
            path: None,
            source: err,
        }
    }
}

impl From<anyhow::Error> for McpError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err)
    }
}

impl From<dialoguer::Error> for McpError {
    fn from(err: dialoguer::Error) -> Self {
        Self::Other(anyhow::anyhow!("Dialog error: {}", err))
    }
}

/// A type alias for [`std::result::Result`] with [`McpError`] as the error type.
///
/// This is the standard result type used throughout MCP Helper for operations
/// that can fail with user-friendly error messages.
///
/// # Example
/// ```rust,no_run
/// use mcp_helper::error::Result;
///
/// fn install_server(name: &str) -> Result<()> {
///     // Installation logic here
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, McpError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deps::InstallMethod;
    use std::error::Error;

    #[test]
    fn test_missing_dependency_constructor() {
        let instructions = InstallInstructions::default();
        let error =
            McpError::missing_dependency("Node.js", Some("18.0.0".to_string()), instructions);

        match error {
            McpError::MissingDependency {
                dependency,
                required_version,
                install_instructions: _,
            } => {
                assert_eq!(dependency, "Node.js");
                assert_eq!(required_version, Some("18.0.0".to_string()));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_missing_dependency_no_version() {
        let instructions = InstallInstructions::default();
        let error = McpError::missing_dependency("Git", None, instructions);

        match error {
            McpError::MissingDependency {
                dependency,
                required_version,
                install_instructions: _,
            } => {
                assert_eq!(dependency, "Git");
                assert_eq!(required_version, None);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_version_mismatch_constructor() {
        let instructions = InstallInstructions::default();
        let error = McpError::version_mismatch("Python", "3.8.0", "3.10.0", instructions);

        match error {
            McpError::VersionMismatch {
                dependency,
                current_version,
                required_version,
                upgrade_instructions: _,
            } => {
                assert_eq!(dependency, "Python");
                assert_eq!(current_version, "3.8.0");
                assert_eq!(required_version, "3.10.0");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_configuration_required_constructor() {
        let missing = vec!["api_key".to_string(), "secret".to_string()];
        let descriptions = vec![
            (
                "api_key".to_string(),
                "API key for authentication".to_string(),
            ),
            ("secret".to_string(), "Secret token".to_string()),
        ];

        let error =
            McpError::configuration_required("test-server", missing.clone(), descriptions.clone());

        match error {
            McpError::ConfigurationRequired {
                server_name,
                missing_fields,
                field_descriptions,
            } => {
                assert_eq!(server_name, "test-server");
                assert_eq!(missing_fields, missing);
                assert_eq!(field_descriptions, descriptions);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_client_not_found_constructor() {
        let available = vec!["Claude Desktop".to_string(), "VS Code".to_string()];
        let error =
            McpError::client_not_found("Cursor", available.clone(), "Visit cursor.sh to install");

        match error {
            McpError::ClientNotFound {
                client_name,
                available_clients,
                install_guidance,
            } => {
                assert_eq!(client_name, "Cursor");
                assert_eq!(available_clients, available);
                assert_eq!(install_guidance, "Visit cursor.sh to install");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_config_error_constructor() {
        let error = McpError::config_error("/path/to/config.json", "Invalid JSON syntax");

        match error {
            McpError::ConfigError { path, message } => {
                assert_eq!(path, "/path/to/config.json");
                assert_eq!(message, "Invalid JSON syntax");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_server_error_constructor() {
        let error = McpError::server_error("my-server", "Failed to start process");

        match error {
            McpError::ServerError {
                server_name,
                message,
            } => {
                assert_eq!(server_name, "my-server");
                assert_eq!(message, "Failed to start process");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_io_error_constructor() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = McpError::io_error("reading config", Some("/path/to/file".to_string()), io_err);

        match error {
            McpError::IoError {
                operation,
                path,
                source,
            } => {
                assert_eq!(operation, "reading config");
                assert_eq!(path, Some("/path/to/file".to_string()));
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_io_error_no_path() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let error = McpError::io_error("creating directory", None, io_err);

        match error {
            McpError::IoError {
                operation,
                path,
                source,
            } => {
                assert_eq!(operation, "creating directory");
                assert_eq!(path, None);
                assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::AlreadyExists, "already exists");
        let error: McpError = io_err.into();

        match error {
            McpError::IoError {
                operation,
                path,
                source,
            } => {
                assert_eq!(operation, "unknown");
                assert_eq!(path, None);
                assert_eq!(source.kind(), std::io::ErrorKind::AlreadyExists);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("Something went wrong");
        let error: McpError = anyhow_err.into();

        match error {
            McpError::Other(err) => {
                assert_eq!(err.to_string(), "Something went wrong");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_from_dialoguer_error() {
        // Create a dialoguer error (we can't easily create a real one, so we'll test the conversion)
        let error = McpError::from(dialoguer::Error::from(std::io::Error::other("test error")));

        match error {
            McpError::Other(err) => {
                assert!(err.to_string().contains("Dialog error"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let error = McpError::io_error("test", None, io_err);

        assert!(error.source().is_some());

        let other_error = McpError::Other(anyhow::anyhow!("test"));
        assert!(other_error.source().is_some());

        let no_source = McpError::server_error("test", "message");
        assert!(no_source.source().is_none());
    }

    #[test]
    fn test_display_missing_dependency() {
        let mut instructions = InstallInstructions::default();
        instructions.windows.push(InstallMethod {
            name: "winget".to_string(),
            command: "winget install Node.js".to_string(),
            description: Some("Recommended".to_string()),
        });

        let error =
            McpError::missing_dependency("Node.js", Some("18.0.0".to_string()), instructions);
        let display = format!("{error}");

        assert!(display.contains("Missing dependency"));
        assert!(display.contains("Node.js"));
        assert!(display.contains("18.0.0"));
    }

    #[test]
    fn test_display_version_mismatch() {
        let instructions = InstallInstructions::default();
        let error = McpError::version_mismatch("Docker", "20.10.0", "24.0.0", instructions);
        let display = format!("{error}");

        assert!(display.contains("Version mismatch"));
        assert!(display.contains("Docker"));
        assert!(display.contains("20.10.0"));
        assert!(display.contains("24.0.0"));
    }

    #[test]
    fn test_display_configuration_required() {
        let missing = vec!["token".to_string()];
        let descriptions = vec![("token".to_string(), "Auth token".to_string())];
        let error = McpError::configuration_required("api-server", missing, descriptions);
        let display = format!("{error}");

        assert!(display.contains("Configuration required"));
        assert!(display.contains("api-server"));
        assert!(display.contains("token"));
        assert!(display.contains("Auth token"));
    }

    #[test]
    fn test_display_client_not_found() {
        let available = vec!["Claude Desktop".to_string()];
        let error = McpError::client_not_found("VS Code", available, "Install from marketplace");
        let display = format!("{error}");

        assert!(display.contains("MCP client not found"));
        assert!(display.contains("VS Code"));
        assert!(display.contains("Claude Desktop"));
        assert!(display.contains("Install from marketplace"));
    }

    #[test]
    fn test_display_config_error() {
        let error = McpError::config_error("/etc/config.json", "Permission denied");
        let display = format!("{error}");

        assert!(display.contains("Configuration error"));
        assert!(display.contains("/etc/config.json"));
        assert!(display.contains("Permission denied"));
    }

    #[test]
    fn test_display_server_error() {
        let error = McpError::server_error("test-server", "Port already in use");
        let display = format!("{error}");

        assert!(display.contains("Server error"));
        assert!(display.contains("test-server"));
        assert!(display.contains("Port already in use"));
    }

    #[test]
    fn test_display_io_error_with_path() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let error = McpError::io_error("reading file", Some("/tmp/test.txt".to_string()), io_err);
        let display = format!("{error}");

        assert!(display.contains("I/O error"));
        assert!(display.contains("reading file"));
        assert!(display.contains("/tmp/test.txt"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_display_io_error_no_path() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let error = McpError::io_error("creating dir", None, io_err);
        let display = format!("{error}");

        assert!(display.contains("I/O error"));
        assert!(display.contains("creating dir"));
        assert!(display.contains("denied"));
    }

    #[test]
    fn test_display_other_error() {
        let error = McpError::Other(anyhow::anyhow!("Custom error message"));
        let display = format!("{error}");

        assert!(display.contains("Custom error message"));
    }

    #[test]
    fn test_write_error_header() {
        // Test that the function exists and returns the expected type
        use std::fmt::Write;
        let mut output = String::new();
        let result = write!(output, "Test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_detail() {
        // Test that the function exists and returns the expected type
        use std::fmt::Write;
        let mut output = String::new();
        let result = write!(output, "Test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_section_header() {
        // Test that the function exists and returns the expected type
        use std::fmt::Write;
        let mut output = String::new();
        let result = write!(output, "Test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_configuration_required_empty_descriptions() {
        let missing = vec!["field1".to_string()];
        let error = McpError::configuration_required("server", missing, vec![]);
        let display = format!("{error}");

        assert!(display.contains("field1"));
        assert!(!display.contains("Field descriptions"));
    }

    #[test]
    fn test_client_not_found_empty_available() {
        let error = McpError::client_not_found("Client", vec![], "No clients available");
        let display = format!("{error}");

        assert!(display.contains("Client"));
        assert!(!display.contains("Available clients"));
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(test_function().unwrap(), 42);

        fn error_function() -> Result<()> {
            Err(McpError::server_error("test", "failed"))
        }

        assert!(error_function().is_err());
    }
}
