use colored::Colorize;
use std::fmt;

use crate::deps::InstallInstructions;

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

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDependency {
                dependency,
                required_version,
                install_instructions,
            } => {
                writeln!(
                    f,
                    "{} Missing dependency: {}",
                    "✗".red().bold(),
                    dependency.yellow()
                )?;
                if let Some(version) = required_version {
                    writeln!(f, "  {} Required version: {}", "→".blue(), version)?;
                }
                writeln!(f)?;
                writeln!(f, "{}", "How to install:".green().bold())?;
                format_install_instructions(f, install_instructions)?;
                Ok(())
            }
            Self::VersionMismatch {
                dependency,
                current_version,
                required_version,
                upgrade_instructions,
            } => {
                writeln!(
                    f,
                    "{} Version mismatch for: {}",
                    "✗".red().bold(),
                    dependency.yellow()
                )?;
                writeln!(
                    f,
                    "  {} Current version: {}",
                    "→".blue(),
                    current_version.red()
                )?;
                writeln!(
                    f,
                    "  {} Required version: {}",
                    "→".blue(),
                    required_version.green()
                )?;
                writeln!(f)?;
                writeln!(f, "{}", "How to upgrade:".green().bold())?;
                format_install_instructions(f, upgrade_instructions)?;
                Ok(())
            }
            Self::ConfigurationRequired {
                server_name,
                missing_fields,
                field_descriptions,
            } => {
                writeln!(
                    f,
                    "{} Configuration required for: {}",
                    "✗".red().bold(),
                    server_name.yellow()
                )?;
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
            Self::ClientNotFound {
                client_name,
                available_clients,
                install_guidance,
            } => {
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
                writeln!(f, "  {install_guidance}")?;
                Ok(())
            }
            Self::ConfigError { path, message } => {
                writeln!(f, "{} Configuration error", "✗".red().bold())?;
                writeln!(f, "  {} Path: {}", "→".blue(), path.yellow())?;
                writeln!(f, "  {} Error: {}", "→".blue(), message)?;
                Ok(())
            }
            Self::ServerError {
                server_name,
                message,
            } => {
                writeln!(
                    f,
                    "{} Server error: {}",
                    "✗".red().bold(),
                    server_name.yellow()
                )?;
                writeln!(f, "  {} {}", "→".blue(), message)?;
                Ok(())
            }
            Self::IoError {
                operation,
                path,
                source,
            } => {
                writeln!(
                    f,
                    "{} I/O error during: {}",
                    "✗".red().bold(),
                    operation.yellow()
                )?;
                if let Some(path) = path {
                    writeln!(f, "  {} Path: {}", "→".blue(), path)?;
                }
                writeln!(f, "  {} Error: {}", "→".blue(), source)?;
                Ok(())
            }
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
