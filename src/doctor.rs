//! MCP diagnostics and troubleshooting command implementation.
//!
//! The doctor command diagnoses common MCP issues and provides actionable
//! solutions. It checks for environment problems, configuration issues,
//! and platform-specific quirks that might prevent MCP servers from working.

use anyhow::Result;
use colored::Colorize;
#[cfg(target_os = "windows")]
use std::collections::HashSet;
use std::process::Command;

use crate::client::detect_clients;
use crate::deps::{DependencyChecker, DockerChecker, NodeChecker};
use crate::error::McpError;

/// Diagnostic check result
#[derive(Debug)]
struct DiagnosticResult {
    category: String,
    check: String,
    status: DiagnosticStatus,
    message: Option<String>,
    solution: Option<String>,
}

#[derive(Debug, PartialEq)]
enum DiagnosticStatus {
    Ok,
    Warning,
    Error,
}

/// MCP Doctor command for diagnostics and troubleshooting
pub struct DoctorCommand {
    verbose: bool,
}

impl DoctorCommand {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn execute(&self) -> Result<(), McpError> {
        println!("{}", "🏥 MCP Doctor - System Diagnostics".blue().bold());
        println!();
        println!("Running comprehensive system checks...");
        println!();

        let mut results = Vec::new();
        let mut has_errors = false;
        let mut has_warnings = false;

        // Check Node.js and npm
        self.check_nodejs(&mut results);

        // Check Docker (optional)
        self.check_docker(&mut results);

        // Check MCP clients
        self.check_clients(&mut results);

        // Check PATH environment
        self.check_path(&mut results);

        // Check platform-specific issues
        self.check_platform_specific(&mut results);

        // Check common server configurations
        self.check_server_configs(&mut results);

        // Display results
        println!("{}", "Diagnostic Results:".blue().bold());
        println!();

        for result in &results {
            let status_symbol = match result.status {
                DiagnosticStatus::Ok => "✓".green(),
                DiagnosticStatus::Warning => "⚠".yellow(),
                DiagnosticStatus::Error => "✗".red(),
            };

            println!(
                "{} {} - {}",
                status_symbol,
                result.category.cyan(),
                result.check
            );

            if let Some(message) = &result.message {
                println!("  {}", message.dimmed());
            }

            if result.status != DiagnosticStatus::Ok {
                if result.status == DiagnosticStatus::Error {
                    has_errors = true;
                } else {
                    has_warnings = true;
                }

                if let Some(solution) = &result.solution {
                    println!("  {} {}", "→ Solution:".green(), solution);
                }
            }
        }

        // Summary
        println!();
        if has_errors {
            println!(
                "{}",
                "❌ Critical issues found that need to be fixed"
                    .red()
                    .bold()
            );
            println!("Please address the errors above before using MCP Helper.");
        } else if has_warnings {
            println!(
                "{}",
                "⚠️  Some warnings found but MCP should work"
                    .yellow()
                    .bold()
            );
            println!("Consider addressing the warnings for optimal performance.");
        } else {
            println!(
                "{}",
                "✅ All checks passed! MCP is ready to use".green().bold()
            );
        }

        println!();
        println!("For more help:");
        println!("  • Run with --verbose for detailed output");
        println!("  • Check documentation at https://github.com/sapientpants/mcp-helper");
        println!("  • Report issues at https://github.com/sapientpants/mcp-helper/issues");

        if has_errors {
            Err(McpError::Other(anyhow::anyhow!(
                "Critical issues found. Please fix them before continuing."
            )))
        } else {
            Ok(())
        }
    }

    fn check_nodejs(&self, results: &mut Vec<DiagnosticResult>) {
        let checker = NodeChecker::new();

        match checker.check() {
            Ok(check) => {
                match check.status {
                    crate::deps::DependencyStatus::Installed { version } => {
                        results.push(DiagnosticResult {
                            category: "Node.js".to_string(),
                            check: format!(
                                "Installation ({})",
                                version.as_deref().unwrap_or("unknown")
                            ),
                            status: DiagnosticStatus::Ok,
                            message: None,
                            solution: None,
                        });

                        // Check npm
                        self.check_command("npm", &["--version"], "npm", results);

                        // Check npx
                        self.check_npx_command(results);
                    }
                    crate::deps::DependencyStatus::Missing => {
                        results.push(DiagnosticResult {
                            category: "Node.js".to_string(),
                            check: "Installation".to_string(),
                            status: DiagnosticStatus::Error,
                            message: Some("Node.js is not installed".to_string()),
                            solution: Some("Install Node.js from https://nodejs.org or use your package manager".to_string()),
                        });
                    }
                    crate::deps::DependencyStatus::VersionMismatch {
                        installed,
                        required,
                    } => {
                        results.push(DiagnosticResult {
                            category: "Node.js".to_string(),
                            check: "Version".to_string(),
                            status: DiagnosticStatus::Warning,
                            message: Some(format!(
                                "Version {installed} installed, {required} recommended"
                            )),
                            solution: Some(
                                "Consider updating Node.js for better compatibility".to_string(),
                            ),
                        });
                    }
                    _ => {}
                }
            }
            Err(e) => {
                results.push(DiagnosticResult {
                    category: "Node.js".to_string(),
                    check: "Detection".to_string(),
                    status: DiagnosticStatus::Error,
                    message: Some(format!("Failed to check Node.js: {e}")),
                    solution: Some("Ensure Node.js is in your PATH".to_string()),
                });
            }
        }
    }

    fn check_docker(&self, results: &mut Vec<DiagnosticResult>) {
        let checker = DockerChecker::new();

        match checker.check() {
            Ok(check) => {
                match check.status {
                    crate::deps::DependencyStatus::Installed { version } => {
                        results.push(DiagnosticResult {
                            category: "Docker".to_string(),
                            check: format!(
                                "Installation ({})",
                                version.as_deref().unwrap_or("unknown")
                            ),
                            status: DiagnosticStatus::Ok,
                            message: Some(
                                "Optional - only needed for Docker-based servers".to_string(),
                            ),
                            solution: None,
                        });
                    }
                    crate::deps::DependencyStatus::Missing => {
                        if self.verbose {
                            results.push(DiagnosticResult {
                                category: "Docker".to_string(),
                                check: "Installation".to_string(),
                                status: DiagnosticStatus::Warning,
                                message: Some("Docker not installed (optional)".to_string()),
                                solution: Some("Install Docker Desktop if you plan to use container-based servers".to_string()),
                            });
                        }
                    }
                    _ => {}
                }
            }
            Err(_) => {
                // Docker check failed, but it's optional so we don't report an error
            }
        }
    }

    fn check_clients(&self, results: &mut Vec<DiagnosticResult>) {
        let clients = detect_clients();
        let installed_clients: Vec<_> = clients
            .iter()
            .filter(|c| c.is_installed())
            .map(|c| c.name())
            .collect();

        if installed_clients.is_empty() {
            results.push(DiagnosticResult {
                category: "MCP Clients".to_string(),
                check: "Installation".to_string(),
                status: DiagnosticStatus::Error,
                message: Some("No MCP clients found".to_string()),
                solution: Some(
                    "Install Claude Desktop, VS Code, or another supported MCP client".to_string(),
                ),
            });
        } else {
            results.push(DiagnosticResult {
                category: "MCP Clients".to_string(),
                check: format!("Found {} client(s)", installed_clients.len()),
                status: DiagnosticStatus::Ok,
                message: Some(installed_clients.join(", ")),
                solution: None,
            });

            // Check for config file access
            for client in &clients {
                if client.is_installed() {
                    let config_path = client.config_path();
                    if config_path.exists() {
                        // Try to read the config
                        match client.list_servers() {
                            Ok(_) => {
                                if self.verbose {
                                    results.push(DiagnosticResult {
                                        category: "Config Access".to_string(),
                                        check: client.name().to_string(),
                                        status: DiagnosticStatus::Ok,
                                        message: None,
                                        solution: None,
                                    });
                                }
                            }
                            Err(e) => {
                                results.push(DiagnosticResult {
                                    category: "Config Access".to_string(),
                                    check: client.name().to_string(),
                                    status: DiagnosticStatus::Warning,
                                    message: Some(format!("Cannot read config: {e}")),
                                    solution: Some(format!(
                                        "Check permissions on {}",
                                        config_path.display()
                                    )),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_path(&self, results: &mut Vec<DiagnosticResult>) {
        // Check if common tools are in PATH
        let tools = vec!["node", "npm", "git"];
        let mut missing = Vec::new();

        for tool in tools {
            if which::which(tool).is_err() {
                missing.push(tool);
            }
        }

        if missing.is_empty() {
            results.push(DiagnosticResult {
                category: "PATH".to_string(),
                check: "Common tools".to_string(),
                status: DiagnosticStatus::Ok,
                message: None,
                solution: None,
            });
        } else {
            results.push(DiagnosticResult {
                category: "PATH".to_string(),
                check: "Common tools".to_string(),
                status: DiagnosticStatus::Warning,
                message: Some(format!("Missing from PATH: {}", missing.join(", "))),
                solution: Some("Add missing tools to your PATH environment variable".to_string()),
            });
        }
    }

    fn check_platform_specific(&self, results: &mut Vec<DiagnosticResult>) {
        #[cfg(target_os = "windows")]
        {
            // Check for npx.cmd
            if which::which("npx.cmd").is_err() && which::which("npx").is_err() {
                results.push(DiagnosticResult {
                    category: "Windows".to_string(),
                    check: "npx.cmd availability".to_string(),
                    status: DiagnosticStatus::Warning,
                    message: Some("npx.cmd not found in PATH".to_string()),
                    solution: Some(
                        "Restart terminal after Node.js installation or run: npm install -g npx"
                            .to_string(),
                    ),
                });
            }

            // Check for common Windows PATH issues
            if let Ok(path) = std::env::var("PATH") {
                let paths: HashSet<_> = path.split(';').collect();

                // Check for npm global bin
                let npm_paths = vec![r"npm\node_modules\.bin", r"nodejs\node_modules\npm\bin"];

                let has_npm_path = npm_paths
                    .iter()
                    .any(|p| paths.iter().any(|path| path.contains(p)));

                if !has_npm_path && self.verbose {
                    results.push(DiagnosticResult {
                        category: "Windows".to_string(),
                        check: "npm global bin in PATH".to_string(),
                        status: DiagnosticStatus::Warning,
                        message: Some("npm global bin directory might not be in PATH".to_string()),
                        solution: Some("Add %APPDATA%\\npm to your PATH".to_string()),
                    });
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Check for shell profile sourcing issues
            if let Ok(home) = std::env::var("HOME") {
                let profiles = [
                    format!("{home}/.zshrc"),
                    format!("{home}/.bash_profile"),
                    format!("{home}/.bashrc"),
                ];

                let has_profile = profiles.iter().any(|p| std::path::Path::new(p).exists());

                if !has_profile && self.verbose {
                    results.push(DiagnosticResult {
                        category: "macOS".to_string(),
                        check: "Shell profile".to_string(),
                        status: DiagnosticStatus::Warning,
                        message: Some("No shell profile found".to_string()),
                        solution: Some(
                            "Create ~/.zshrc or ~/.bash_profile for PATH configuration".to_string(),
                        ),
                    });
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Check for snap vs system Node.js conflicts
            if which::which("node").is_ok() {
                if let Ok(output) = Command::new("which").arg("node").output() {
                    let path = String::from_utf8_lossy(&output.stdout);
                    if path.contains("/snap/") && self.verbose {
                        results.push(DiagnosticResult {
                            category: "Linux".to_string(),
                            check: "Node.js installation".to_string(),
                            status: DiagnosticStatus::Warning,
                            message: Some("Node.js installed via snap".to_string()),
                            solution: Some("Snap Node.js can have permission issues. Consider using NodeSource or nvm".to_string()),
                        });
                    }
                }
            }
        }
    }

    fn check_server_configs(&self, results: &mut Vec<DiagnosticResult>) {
        // Check for common server configuration issues
        let clients = detect_clients();
        let mut total_servers = 0;
        let mut servers_with_issues = Vec::new();

        for client in &clients {
            if !client.is_installed() {
                continue;
            }

            if let Ok(servers) = client.list_servers() {
                total_servers += servers.len();

                for (name, config) in servers {
                    // Check for common issues
                    if config.command.is_empty() {
                        servers_with_issues.push(format!("{name} (empty command)"));
                    } else if config.command == "npx" && cfg!(target_os = "windows") {
                        // On Windows, npx might need to be npx.cmd
                        if which::which("npx").is_err() && which::which("npx.cmd").is_ok() {
                            servers_with_issues
                                .push(format!("{name} (should use npx.cmd on Windows)"));
                        }
                    }
                }
            }
        }

        if total_servers > 0 {
            if servers_with_issues.is_empty() {
                results.push(DiagnosticResult {
                    category: "Server Configs".to_string(),
                    check: format!("{total_servers} server(s) configured"),
                    status: DiagnosticStatus::Ok,
                    message: None,
                    solution: None,
                });
            } else {
                results.push(DiagnosticResult {
                    category: "Server Configs".to_string(),
                    check: "Configuration issues".to_string(),
                    status: DiagnosticStatus::Warning,
                    message: Some(format!("Issues found: {}", servers_with_issues.join(", "))),
                    solution: Some("Run 'mcp config list' to review configurations".to_string()),
                });
            }
        }
    }

    fn check_command(
        &self,
        command: &str,
        args: &[&str],
        name: &str,
        results: &mut Vec<DiagnosticResult>,
    ) {
        match Command::new(command).args(args).output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                results.push(DiagnosticResult {
                    category: name.to_string(),
                    check: format!("Installation ({version})"),
                    status: DiagnosticStatus::Ok,
                    message: None,
                    solution: None,
                });
            }
            _ => {
                results.push(DiagnosticResult {
                    category: name.to_string(),
                    check: "Installation".to_string(),
                    status: DiagnosticStatus::Warning,
                    message: Some(format!("{name} not found or not working")),
                    solution: Some(format!("Ensure {name} is installed and in PATH")),
                });
            }
        }
    }

    fn check_npx_command(&self, results: &mut Vec<DiagnosticResult>) {
        let npx_cmd = if cfg!(target_os = "windows") {
            "npx.cmd"
        } else {
            "npx"
        };

        match Command::new(npx_cmd).arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                results.push(DiagnosticResult {
                    category: "npx".to_string(),
                    check: format!("Installation ({version}) [{npx_cmd}]"),
                    status: DiagnosticStatus::Ok,
                    message: None,
                    solution: None,
                });
            }
            _ => {
                results.push(DiagnosticResult {
                    category: "npx".to_string(),
                    check: "Installation".to_string(),
                    status: DiagnosticStatus::Warning,
                    message: Some(format!("{npx_cmd} not found")),
                    solution: Some(
                        "npx will be downloaded on first use, or run: npm install -g npx"
                            .to_string(),
                    ),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_command_creation() {
        let doctor = DoctorCommand::new(false);
        assert!(!doctor.verbose);

        let doctor = DoctorCommand::new(true);
        assert!(doctor.verbose);
    }

    #[test]
    fn test_diagnostic_status() {
        assert_ne!(DiagnosticStatus::Ok, DiagnosticStatus::Warning);
        assert_ne!(DiagnosticStatus::Ok, DiagnosticStatus::Error);
        assert_ne!(DiagnosticStatus::Warning, DiagnosticStatus::Error);
    }
}
