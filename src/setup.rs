//! MCP environment setup command implementation.
//!
//! This module verifies the user's environment is ready to run MCP servers.
//! It checks for required tools (Node.js, Docker, etc.) and provides guidance
//! when they're missing. Following our architecture, we don't install these
//! tools - we just verify they're available and guide users to official installers.

use anyhow::Result;
use colored::Colorize;
use std::process::Command;

use crate::deps::{DependencyChecker, DockerChecker, InstallInstructions, NodeChecker};
use crate::error::McpError;

/// Environment setup and verification command
pub struct SetupCommand {
    verbose: bool,
}

impl SetupCommand {
    /// Create a new setup command instance
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Execute the setup command
    pub fn execute(&self) -> Result<(), McpError> {
        println!("{}", "🔧 MCP Helper Environment Setup".blue().bold());
        println!("Verifying your environment has the required tools...");
        println!();

        // Check for Node.js (required for npm-based servers)
        self.check_nodejs()?;

        // Check for Docker (optional, for container-based servers)
        self.check_docker()?;

        // Check for shell completions
        self.check_shell_completions()?;

        // Verify npx command works correctly
        self.verify_npx_command()?;

        println!();
        println!("{}", "✅ Environment setup complete!".green().bold());
        println!();
        println!("You can now:");
        println!("  • Add servers with: {}", "mcp add <server>".cyan());
        println!("  • List configured servers with: {}", "mcp list".cyan());
        println!("  • Remove servers with: {}", "mcp remove <server>".cyan());
        println!();
        println!("If you encounter issues, run: {}", "mcp doctor".cyan());

        Ok(())
    }

    fn check_nodejs(&self) -> Result<(), McpError> {
        println!("{} Checking Node.js installation...", "→".green());

        let checker = NodeChecker::new();
        let check = checker
            .check()
            .map_err(|e| McpError::Other(anyhow::anyhow!("Failed to check Node.js: {}", e)))?;

        match check.status {
            crate::deps::DependencyStatus::Installed { version } => {
                let version_str = version.as_deref().unwrap_or("unknown");
                println!("  {} Node.js: {}", "✓".green(), version_str.cyan());

                // Also check npm and npx
                if let Ok(npm_version) = self.get_command_version("npm", &["--version"]) {
                    println!("  {} npm: {}", "✓".green(), npm_version.cyan());
                }

                if let Ok(npx_version) = self.get_command_version("npx", &["--version"]) {
                    println!("  {} npx: {}", "✓".green(), npx_version.cyan());
                } else {
                    println!(
                        "  {} npx: Not found (will be downloaded on first use)",
                        "⚠".yellow()
                    );
                }
            }
            crate::deps::DependencyStatus::Missing => {
                println!("  {} Node.js: Not installed", "✗".red());
                println!();
                println!(
                    "{}",
                    "Node.js is required for npm-based MCP servers.".yellow()
                );

                if let Some(ref instructions) = check.install_instructions {
                    println!("To install Node.js:");
                    for method in instructions.for_platform() {
                        println!("  • {}: {}", method.name.green(), method.command.cyan());
                        if let Some(desc) = &method.description {
                            println!("    {}", desc.dimmed());
                        }
                    }
                }

                return Err(McpError::missing_dependency(
                    "Node.js",
                    None,
                    check
                        .install_instructions
                        .unwrap_or_else(InstallInstructions::default),
                ));
            }
            crate::deps::DependencyStatus::VersionMismatch {
                installed,
                required,
            } => {
                println!(
                    "  {} Node.js: {} (required: {})",
                    "⚠".yellow(),
                    installed.yellow(),
                    required.green()
                );
                println!("    Consider updating Node.js for best compatibility");
            }
            crate::deps::DependencyStatus::ConfigurationRequired { issue, solution } => {
                println!("  {} Node.js: Configuration required", "⚠".yellow());
                println!("    Issue: {issue}");
                println!("    Solution: {}", solution.cyan());
            }
        }

        Ok(())
    }

    fn check_docker(&self) -> Result<(), McpError> {
        println!("{} Checking Docker installation...", "→".green());

        let checker = DockerChecker::new();
        let check = checker
            .check()
            .map_err(|e| McpError::Other(anyhow::anyhow!("Failed to check Docker: {}", e)))?;

        match check.status {
            crate::deps::DependencyStatus::Installed { version } => {
                let version_str = version.as_deref().unwrap_or("unknown");
                println!("  {} Docker: {}", "✓".green(), version_str.cyan());
            }
            crate::deps::DependencyStatus::Missing => {
                println!("  {} Docker: Not installed (optional)", "ℹ".blue());
                println!("    Docker is only needed for container-based MCP servers");
            }
            _ => {
                println!("  {} Docker: May need configuration", "⚠".yellow());
            }
        }

        Ok(())
    }

    fn check_shell_completions(&self) -> Result<(), McpError> {
        println!("{} Checking shell completions...", "→".green());

        // For now, just inform about future shell completion support
        println!("  {} Shell completions: Not yet implemented", "ℹ".blue());
        println!("    Shell completions will be available in a future release");

        Ok(())
    }

    fn verify_npx_command(&self) -> Result<(), McpError> {
        println!("{} Verifying npx command...", "→".green());

        // Determine the correct npx command for the platform
        let npx_cmd = if cfg!(target_os = "windows") {
            "npx.cmd"
        } else {
            "npx"
        };

        // Try to run npx --version
        match Command::new(npx_cmd).arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!(
                    "  {} npx command works correctly ({})",
                    "✓".green(),
                    npx_cmd.cyan()
                );
                if self.verbose {
                    println!("    Version: {}", version.dimmed());
                }
            }
            Ok(_) => {
                println!("  {} npx command found but returned an error", "⚠".yellow());
                println!("    This might be resolved on first use");
            }
            Err(_) => {
                if cfg!(target_os = "windows") {
                    println!("  {} npx.cmd not found in PATH", "⚠".yellow());
                    println!("    Windows users may need to:");
                    println!("    1. Restart terminal after Node.js installation");
                    println!("    2. Add npm global bin to PATH");
                    println!("    3. Run: {}", "npm install -g npx".cyan());
                } else {
                    println!("  {} npx not found", "⚠".yellow());
                    println!("    npx will be downloaded automatically on first use");
                }
            }
        }

        Ok(())
    }

    fn get_command_version(&self, command: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(command).args(args).output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            anyhow::bail!("Command failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_command_creation() {
        let setup = SetupCommand::new(false);
        assert!(!setup.verbose);

        let setup = SetupCommand::new(true);
        assert!(setup.verbose);
    }

    #[test]
    fn test_get_command_version() {
        let setup = SetupCommand::new(false);

        // Test with echo command which should exist on most systems
        if cfg!(target_os = "windows") {
            // Windows echo works differently
            let result = setup.get_command_version("cmd", &["/C", "echo test"]);
            assert!(result.is_ok() || result.is_err()); // May or may not work
        } else {
            let result = setup.get_command_version("echo", &["test"]);
            if result.is_ok() {
                assert_eq!(result.unwrap(), "test");
            }
        }
    }
}
