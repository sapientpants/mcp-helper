//! Add command implementation for MCP Helper.
//!
//! This module implements the unified `mcp add` command that combines smart
//! server detection with manual configuration options. It automatically handles
//! platform-specific differences like npx vs npx.cmd on Windows.

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect};
use std::collections::HashMap;

use crate::client::{detect_clients, McpClient, ServerConfig};
use crate::deps::{DependencyChecker, NodeChecker};
use crate::error::McpError;
use crate::server::{detect_server_type, ServerType};

/// Add command for configuring MCP servers
pub struct AddCommand {
    verbose: bool,
}

impl AddCommand {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn execute(
        &mut self,
        server: &str,
        command: Option<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
        non_interactive: bool,
    ) -> Result<(), McpError> {
        println!("{} Adding MCP server: {}", "→".green(), server.cyan());
        println!();

        // Detect installed clients
        let clients = detect_clients();
        let installed_clients: Vec<&dyn McpClient> = clients
            .iter()
            .filter(|c| c.is_installed())
            .map(|c| c.as_ref())
            .collect();

        if installed_clients.is_empty() {
            return Err(McpError::Other(anyhow::anyhow!(
                "No MCP clients found. Please install Claude Desktop, VS Code, or another supported client."
            )));
        }

        // Try to detect server type if command not specified
        let (final_command, final_args, server_name) = if command.is_none() {
            self.detect_server_config(server, args)?
        } else {
            // Manual configuration
            let cmd = self.get_platform_command(&command.unwrap());
            (cmd, args, server.to_string())
        };

        // Check dependencies based on command type
        self.check_dependencies(&final_command)?;

        // Build the configuration
        let mut config = ServerConfig {
            command: final_command.clone(),
            args: final_args.clone(),
            env: env.clone(),
        };

        // Add any additional configuration if interactive
        if !non_interactive {
            config = self.configure_interactively(config)?;
        }

        // Select which clients to add to
        let selected_clients = if non_interactive {
            // Add to all clients in non-interactive mode
            installed_clients
        } else {
            self.select_clients(&installed_clients)?
        };

        if selected_clients.is_empty() {
            println!("{} No clients selected", "❌".red());
            return Ok(());
        }

        // Show preview
        self.show_preview(&server_name, &config, &selected_clients);

        // Confirm if interactive
        if !non_interactive {
            let confirm = Confirm::new()
                .with_prompt("Add this server configuration?")
                .default(true)
                .interact()
                .map_err(|e| McpError::Other(anyhow::anyhow!("Confirmation failed: {}", e)))?;

            if !confirm {
                println!("{} Configuration cancelled", "❌".red());
                return Ok(());
            }
        }

        // Add to selected clients
        let mut success_count = 0;
        let mut errors = Vec::new();

        for client in selected_clients {
            match client.add_server(&server_name, config.clone()) {
                Ok(_) => {
                    success_count += 1;
                    if self.verbose {
                        println!("  {} Added to {}", "✓".green(), client.name().cyan());
                    }
                }
                Err(e) => {
                    errors.push((client.name(), e));
                }
            }
        }

        // Report results
        println!();
        if success_count > 0 {
            println!(
                "{} Server '{}' added to {} client(s)",
                "✅".green(),
                server_name.cyan(),
                success_count
            );
        }

        if !errors.is_empty() {
            println!();
            println!("{} Failed to add to some clients:", "⚠".yellow());
            for (client, error) in errors {
                println!("  • {}: {}", client, error.to_string().dimmed());
            }
        }

        Ok(())
    }

    fn detect_server_config(
        &self,
        server: &str,
        mut args: Vec<String>,
    ) -> Result<(String, Vec<String>, String), McpError> {
        // Try to detect server type
        let server_type = detect_server_type(server);

        if self.verbose {
            println!("Detected server type: {server_type:?}");
        }

        match server_type {
            ServerType::Npm { package, version } => {
                // For NPM packages, use npx (or npx.cmd on Windows)
                let command = self.get_platform_command("npx");

                // Build args: package name with version if specified
                let package_arg = if let Some(v) = version {
                    format!("{package}@{v}")
                } else {
                    package.clone()
                };

                // If no args provided, just use the package
                if args.is_empty() {
                    args = vec![package_arg];
                } else {
                    // Prepend package to existing args
                    args.insert(0, package_arg);
                }

                // Extract clean server name for configuration
                let server_name = package
                    .split('/')
                    .next_back()
                    .unwrap_or(&package)
                    .to_string();

                Ok((command, args, server_name))
            }
            ServerType::Binary { url, .. } => {
                // For binary servers, download and use the binary
                // For now, just use the URL as-is (future: download logic)
                println!("{} Binary server support coming soon", "⚠".yellow());
                println!("Using URL as command: {url}");
                Ok((url.clone(), args, server.to_string()))
            }
            ServerType::Docker { image, tag } => {
                // For Docker images, use docker run
                let command = "docker".to_string();
                let mut docker_args = vec!["run".to_string(), "--rm".to_string(), "-i".to_string()];

                let full_image = if let Some(t) = tag {
                    format!("{image}:{t}")
                } else {
                    image.clone()
                };

                docker_args.push(full_image);
                docker_args.extend(args);

                let server_name = image.split('/').next_back().unwrap_or(&image).to_string();

                Ok((command, docker_args, server_name))
            }
            ServerType::Python { package, version } => {
                // For Python packages, use python -m
                let command = "python".to_string();
                let mut python_args = vec!["-m".to_string()];

                // Add package with version if specified
                if let Some(v) = version {
                    python_args.push(format!("{package}=={v}"));
                } else {
                    python_args.push(package.clone());
                }

                python_args.extend(args);

                let server_name = package
                    .split('/')
                    .next_back()
                    .unwrap_or(&package)
                    .to_string();

                Ok((command, python_args, server_name))
            }
        }
    }

    fn get_platform_command(&self, command: &str) -> String {
        // Handle platform-specific command variations
        if command == "npx" && cfg!(target_os = "windows") {
            // On Windows, prefer npx.cmd if available
            if which::which("npx.cmd").is_ok() {
                "npx.cmd".to_string()
            } else {
                "npx".to_string()
            }
        } else {
            command.to_string()
        }
    }

    fn check_dependencies(&self, command: &str) -> Result<(), McpError> {
        // Check dependencies based on command type
        if command == "npx" || command == "npx.cmd" || command == "npm" {
            let checker = NodeChecker::new();
            match checker.check() {
                Ok(check) => {
                    if let crate::deps::DependencyStatus::Missing = check.status {
                        return Err(McpError::Other(anyhow::anyhow!(
                            "Node.js is required for NPM-based servers. Please install from https://nodejs.org"
                        )));
                    }
                }
                Err(e) => {
                    if self.verbose {
                        println!("{} Warning: Could not verify Node.js: {}", "⚠".yellow(), e);
                    }
                }
            }
        }

        // Docker, Python, etc. checks could be added here

        Ok(())
    }

    fn configure_interactively(&self, mut config: ServerConfig) -> Result<ServerConfig, McpError> {
        // Ask if user wants to add environment variables
        let add_env = Confirm::new()
            .with_prompt("Add environment variables?")
            .default(false)
            .interact()
            .map_err(|e| McpError::Other(anyhow::anyhow!("Input failed: {}", e)))?;

        if add_env {
            loop {
                let key: String = Input::new()
                    .with_prompt("Environment variable name (or press Enter to finish)")
                    .allow_empty(true)
                    .interact()
                    .map_err(|e| McpError::Other(anyhow::anyhow!("Input failed: {}", e)))?;

                if key.is_empty() {
                    break;
                }

                let value: String = Input::new()
                    .with_prompt(format!("Value for {key}"))
                    .interact()
                    .map_err(|e| McpError::Other(anyhow::anyhow!("Input failed: {}", e)))?;

                config.env.insert(key, value);
            }
        }

        Ok(config)
    }

    fn select_clients<'a>(
        &self,
        installed_clients: &[&'a dyn McpClient],
    ) -> Result<Vec<&'a dyn McpClient>, McpError> {
        let client_names: Vec<_> = installed_clients.iter().map(|c| c.name()).collect();

        if client_names.len() == 1 {
            // Only one client, select it automatically
            Ok(installed_clients.to_vec())
        } else {
            // Multiple clients, let user choose
            let selections = MultiSelect::new()
                .with_prompt("Select MCP clients to configure")
                .items(&client_names)
                .interact()
                .map_err(|e| McpError::Other(anyhow::anyhow!("Selection failed: {}", e)))?;

            Ok(selections
                .into_iter()
                .map(|i| installed_clients[i])
                .collect())
        }
    }

    fn show_preview(&self, server_name: &str, config: &ServerConfig, clients: &[&dyn McpClient]) {
        println!();
        println!("{}", "Configuration preview:".blue());
        println!("  Server: {}", server_name.cyan());
        println!(
            "  Command: {} {}",
            config.command.green(),
            config.args.join(" ").dimmed()
        );

        if !config.env.is_empty() {
            println!("  Environment:");
            for (key, value) in &config.env {
                println!("    {}: {}", key.cyan(), value);
            }
        }

        println!(
            "  Clients: {}",
            clients
                .iter()
                .map(|c| c.name())
                .collect::<Vec<_>>()
                .join(", ")
                .yellow()
        );
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_command_creation() {
        let cmd = AddCommand::new(false);
        assert!(!cmd.verbose);

        let cmd = AddCommand::new(true);
        assert!(cmd.verbose);
    }

    #[test]
    fn test_platform_command_detection() {
        let cmd = AddCommand::new(false);

        // Test non-npx commands pass through
        assert_eq!(cmd.get_platform_command("python"), "python");
        assert_eq!(cmd.get_platform_command("docker"), "docker");

        // Test npx handling
        #[cfg(not(target_os = "windows"))]
        assert_eq!(cmd.get_platform_command("npx"), "npx");

        // On Windows, it depends on what's available
        #[cfg(target_os = "windows")]
        {
            let result = cmd.get_platform_command("npx");
            assert!(result == "npx" || result == "npx.cmd");
        }
    }
}
