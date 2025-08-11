//! Configuration management commands for MCP Helper.
//!
//! This module implements the config subcommands: list, add, and remove.
//! These commands manage server configurations across different MCP clients.

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use std::collections::HashMap;

use crate::client::{detect_clients, ServerConfig};
use crate::error::McpError;

/// List all configured servers across all MCP clients
pub struct ConfigListCommand {
    verbose: bool,
}

impl ConfigListCommand {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn execute(&self) -> Result<(), McpError> {
        println!("{}", "📋 MCP Server Configurations".blue().bold());
        println!();

        let clients = detect_clients();
        let mut found_any = false;
        let mut total_servers = 0;

        for client in &clients {
            if !client.is_installed() {
                continue;
            }

            let config_path = client.config_path();

            match client.list_servers() {
                Ok(servers) if !servers.is_empty() => {
                    found_any = true;
                    total_servers += servers.len();

                    println!(
                        "{} {} ({})",
                        "→".green(),
                        client.name().cyan().bold(),
                        config_path.display().to_string().dimmed()
                    );

                    for (name, config) in servers.iter() {
                        println!(
                            "  • {}: {} {}",
                            name.yellow(),
                            config.command.green(),
                            config.args.join(" ").dimmed()
                        );

                        if self.verbose && !config.env.is_empty() {
                            println!("    Environment:");
                            for (key, value) in &config.env {
                                println!("      {}: {}", key.cyan(), value);
                            }
                        }
                    }
                    println!();
                }
                Ok(_) => {
                    // Client installed but no servers configured
                    if self.verbose {
                        println!(
                            "{} {} (no servers configured)",
                            "→".dimmed(),
                            client.name().dimmed()
                        );
                    }
                }
                Err(e) => {
                    if self.verbose {
                        println!(
                            "{} {} - Error reading config: {}",
                            "⚠".yellow(),
                            client.name(),
                            e.to_string().dimmed()
                        );
                    }
                }
            }
        }

        if !found_any {
            println!("No MCP servers configured yet.");
            println!();
            println!("To configure a server, run:");
            println!("  {}", "mcp install <server>".cyan());
            println!("  {}", "mcp config add <server>".cyan());
        } else {
            println!(
                "Total: {} server(s) configured",
                total_servers.to_string().green()
            );
        }

        Ok(())
    }
}

/// Add a server to MCP client configuration
pub struct ConfigAddCommand {
    #[allow(dead_code)]
    verbose: bool,
}

impl ConfigAddCommand {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn execute(&self, server_name: &str) -> Result<(), McpError> {
        println!("{} Adding server: {}", "→".green(), server_name.cyan());
        println!();

        // Detect installed clients
        let clients = detect_clients();
        let installed_clients: Vec<_> = clients.iter().filter(|c| c.is_installed()).collect();

        if installed_clients.is_empty() {
            return Err(McpError::Other(anyhow::anyhow!(
                "No MCP clients found. Please install Claude Desktop, VS Code, or another supported client."
            )));
        }

        // Select which client to add to
        let client_names: Vec<_> = installed_clients.iter().map(|c| c.name()).collect();
        let selection = if client_names.len() == 1 {
            0
        } else {
            Select::new()
                .with_prompt("Select MCP client to configure")
                .items(&client_names)
                .default(0)
                .interact()
                .map_err(|e| McpError::Other(anyhow::anyhow!("Selection failed: {}", e)))?
        };

        let selected_client = &installed_clients[selection];
        println!("Configuring for: {}", selected_client.name().cyan());
        println!();

        // Get command details
        let command: String = Input::new()
            .with_prompt("Command to run (e.g., npx, python, docker)")
            .default("npx".to_string())
            .interact()
            .map_err(|e| McpError::Other(anyhow::anyhow!("Input failed: {}", e)))?;

        let args_str: String = Input::new()
            .with_prompt("Arguments (space-separated)")
            .default(server_name.to_string())
            .interact()
            .map_err(|e| McpError::Other(anyhow::anyhow!("Input failed: {}", e)))?;

        let args: Vec<String> = args_str.split_whitespace().map(String::from).collect();

        // Get environment variables
        let mut env = HashMap::new();
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

                env.insert(key, value);
            }
        }

        // Create the server config
        let config = ServerConfig { command, args, env };

        // Show preview
        println!();
        println!("{}", "Configuration preview:".blue());
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
        println!();

        // Confirm
        let confirm = Confirm::new()
            .with_prompt("Add this server configuration?")
            .default(true)
            .interact()
            .map_err(|e| McpError::Other(anyhow::anyhow!("Confirmation failed: {}", e)))?;

        if !confirm {
            println!("{} Configuration cancelled", "❌".red());
            return Ok(());
        }

        // Add the server
        selected_client
            .add_server(server_name, config)
            .map_err(|e| McpError::Other(anyhow::anyhow!("Failed to add server: {}", e)))?;

        println!(
            "{} Server '{}' added to {}",
            "✅".green(),
            server_name.cyan(),
            selected_client.name()
        );

        Ok(())
    }
}

/// Remove a server from MCP client configuration
pub struct ConfigRemoveCommand {
    #[allow(dead_code)]
    verbose: bool,
    remove_all: bool,
}

impl ConfigRemoveCommand {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            remove_all: false,
        }
    }

    pub fn set_remove_all(&mut self, remove_all: bool) {
        self.remove_all = remove_all;
    }

    pub fn execute(&self, server_name: &str) -> Result<(), McpError> {
        println!("{} Removing server: {}", "→".green(), server_name.cyan());
        println!();

        // Detect installed clients
        let clients = detect_clients();
        let mut found_in_clients = Vec::new();

        // Find which clients have this server
        for client in &clients {
            if !client.is_installed() {
                continue;
            }

            if let Ok(servers) = client.list_servers() {
                if servers.contains_key(server_name) {
                    found_in_clients.push(client.as_ref());
                }
            }
        }

        if found_in_clients.is_empty() {
            return Err(McpError::Other(anyhow::anyhow!(
                "Server '{}' not found in any MCP client configuration",
                server_name
            )));
        }

        // If remove_all is set, remove from all clients
        let selected_clients = if self.remove_all {
            found_in_clients.clone()
        } else if found_in_clients.len() == 1 {
            vec![found_in_clients[0]]
        } else {
            // Multiple clients and not remove_all, ask which one
            println!("Server found in multiple clients:");
            let client_names: Vec<_> = found_in_clients.iter().map(|c| c.name()).collect();

            let selection = Select::new()
                .with_prompt("Select client to remove from")
                .items(&client_names)
                .interact()
                .map_err(|e| McpError::Other(anyhow::anyhow!("Selection failed: {}", e)))?;

            vec![found_in_clients[selection]]
        };

        // Show what will be removed
        println!("{}", "Will remove:".yellow());
        for client in &selected_clients {
            if let Ok(servers) = client.list_servers() {
                if let Some(config) = servers.get(server_name) {
                    println!("  Client: {}", client.name().cyan());
                    println!(
                        "    Command: {} {}",
                        config.command.green(),
                        config.args.join(" ").dimmed()
                    );
                }
            }
        }
        println!();

        // Confirm removal
        let prompt = if selected_clients.len() > 1 {
            format!(
                "Remove this server from {} clients?",
                selected_clients.len()
            )
        } else {
            "Remove this server configuration?".to_string()
        };

        let confirm = Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()
            .map_err(|e| McpError::Other(anyhow::anyhow!("Confirmation failed: {}", e)))?;

        if !confirm {
            println!("{} Removal cancelled", "❌".red());
            return Ok(());
        }

        // For now, we'll need to implement remove_server in the McpClient trait
        // As a workaround, we inform the user to manually edit
        println!(
            "{} Note: Server removal requires manual config editing",
            "⚠".yellow()
        );
        println!("This feature will be improved in a future update.");
        println!();

        if selected_clients.len() == 1 {
            println!(
                "{} Server '{}' marked for removal from {}",
                "✅".green(),
                server_name.cyan(),
                selected_clients[0].name()
            );
            println!();
            println!("To complete removal, manually edit:");
            println!(
                "  {}",
                selected_clients[0]
                    .config_path()
                    .display()
                    .to_string()
                    .cyan()
            );
        } else {
            println!(
                "{} Server '{}' marked for removal from {} clients",
                "✅".green(),
                server_name.cyan(),
                selected_clients.len()
            );
            println!();
            println!("To complete removal, manually edit:");
            for client in selected_clients {
                println!(
                    "  • {}: {}",
                    client.name(),
                    client.config_path().display().to_string().cyan()
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_list_command_creation() {
        let cmd = ConfigListCommand::new(false);
        assert!(!cmd.verbose);

        let cmd = ConfigListCommand::new(true);
        assert!(cmd.verbose);
    }

    #[test]
    fn test_config_add_command_creation() {
        let cmd = ConfigAddCommand::new(false);
        assert!(!cmd.verbose);

        let cmd = ConfigAddCommand::new(true);
        assert!(cmd.verbose);
    }

    #[test]
    fn test_config_remove_command_creation() {
        let cmd = ConfigRemoveCommand::new(false);
        assert!(!cmd.verbose);
        assert!(!cmd.remove_all);

        let cmd = ConfigRemoveCommand::new(true);
        assert!(cmd.verbose);
        assert!(!cmd.remove_all);
    }

    #[test]
    fn test_config_remove_set_remove_all() {
        let mut cmd = ConfigRemoveCommand::new(false);
        assert!(!cmd.remove_all);

        cmd.set_remove_all(true);
        assert!(cmd.remove_all);
    }
}
