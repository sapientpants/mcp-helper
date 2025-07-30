use colored::Colorize;
use std::fmt;

use crate::deps::InstallInstructions;

pub mod builder;
pub use builder::ErrorBuilder;

#[derive(Debug)]
pub enum McpError {
    MissingDependency {
        dependency: String,
        required_version: Option<String>,
        install_instructions: Box<InstallInstructions>,
    },
    VersionMismatch {
        dependency: String,
        current_version: String,
        required_version: String,
        upgrade_instructions: Box<InstallInstructions>,
    },
    ConfigurationRequired {
        server_name: String,
        missing_fields: Vec<String>,
        field_descriptions: Vec<(String, String)>,
    },
    ClientNotFound {
        client_name: String,
        available_clients: Vec<String>,
        install_guidance: String,
    },
    ConfigError {
        path: String,
        message: String,
    },
    ServerError {
        server_name: String,
        message: String,
    },
    IoError {
        operation: String,
        path: Option<String>,
        source: std::io::Error,
    },
    Other(anyhow::Error),
}

impl McpError {
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

    pub fn config_error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConfigError {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn server_error(server_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ServerError {
            server_name: server_name.into(),
            message: message.into(),
        }
    }

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

pub type Result<T> = std::result::Result<T, McpError>;
