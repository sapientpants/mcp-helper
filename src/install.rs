use anyhow::{anyhow, Context, Result};
use colored::*;
use dialoguer::{Confirm, Input};
use std::collections::HashMap;

use crate::client::{detect_clients, ClientRegistry, ServerConfig};
use crate::deps::{Dependency, DependencyStatus};
use crate::server::{detect_server_type, ConfigFieldType, McpServer, ServerType};

pub struct InstallCommand {
    client_registry: ClientRegistry,
    verbose: bool,
}

impl InstallCommand {
    pub fn new(verbose: bool) -> Self {
        let mut client_registry = ClientRegistry::new();

        // Register clients
        for client in detect_clients() {
            client_registry.register(client);
        }

        Self {
            client_registry,
            verbose,
        }
    }

    pub fn execute(&self, server_name: &str) -> Result<()> {
        if self.verbose {
            eprintln!("{} Detecting server type for: {}", "ℹ".blue(), server_name);
        }

        // Parse server argument and detect type
        let server_type = detect_server_type(server_name);

        // Create appropriate server instance
        let server = self.create_server(&server_type)?;

        // Run dependency checks
        self.check_dependencies(&*server)?;

        // Select target client(s)
        let clients = self.select_clients()?;

        if clients.is_empty() {
            return Err(anyhow!("No MCP clients selected for installation"));
        }

        // Prompt for configuration
        let config = self.prompt_configuration(&*server)?;

        // Apply configuration to selected clients
        for client_name in &clients {
            self.install_to_client(client_name, server_name, &config)?;
        }

        println!(
            "\n{} Successfully installed {} to {} client(s)",
            "✓".green().bold(),
            server_name.cyan(),
            clients.len()
        );

        Ok(())
    }

    fn create_server(&self, server_type: &ServerType) -> Result<Box<dyn McpServer>> {
        match server_type {
            ServerType::Npm { package, version } => {
                use crate::server::npm::NpmServer;
                Ok(Box::new(NpmServer::from_package(
                    package.clone(),
                    version.clone(),
                )))
            }
            _ => Err(anyhow!(
                "Server type {:?} is not yet supported. Only NPM servers are currently implemented.",
                server_type
            )),
        }
    }

    fn check_dependencies(&self, server: &dyn McpServer) -> Result<()> {
        println!("{} Checking dependencies...", "🔍".blue());

        let dependency = server.dependency();
        let check = dependency.check()?;

        let dep_name = match &check.dependency {
            Dependency::NodeJs { .. } => "Node.js",
            Dependency::Python { .. } => "Python",
            Dependency::Docker => "Docker",
            Dependency::Git => "Git",
        };

        match &check.status {
            DependencyStatus::Installed { version } => {
                println!(
                    "  {} {} is installed{}",
                    "✓".green(),
                    dep_name,
                    if let Some(v) = version {
                        format!(" (version {v})")
                    } else {
                        String::new()
                    }
                );
                Ok(())
            }
            DependencyStatus::Missing => {
                println!("  {} {} is not installed", "✗".red(), dep_name);
                self.show_install_instructions(&check)?;
                Err(anyhow!("Required dependency {} is not installed", dep_name))
            }
            DependencyStatus::VersionMismatch {
                installed,
                required,
            } => {
                println!(
                    "  {} {} version mismatch: found {}, requires {}",
                    "✗".red(),
                    dep_name,
                    installed,
                    required
                );
                self.show_install_instructions(&check)?;
                Err(anyhow!("Dependency {} version mismatch", dep_name))
            }
        }
    }

    fn show_install_instructions(&self, check: &crate::deps::DependencyCheck) -> Result<()> {
        if let Some(instructions) = &check.install_instructions {
            println!("\n{}", "To install this dependency:".yellow());

            #[cfg(target_os = "windows")]
            let methods = &instructions.windows;
            #[cfg(target_os = "macos")]
            let methods = &instructions.macos;
            #[cfg(target_os = "linux")]
            let methods = &instructions.linux;

            for method in methods {
                println!("  Using {}:", method.name.yellow());
                println!("    {}", method.command.bright_white());
                if let Some(desc) = &method.description {
                    println!("    {}: {}", "Note".yellow(), desc);
                }
            }
        }
        Ok(())
    }

    fn select_clients(&self) -> Result<Vec<String>> {
        let installed_clients: Vec<String> = self
            .client_registry
            .detect_installed()
            .into_iter()
            .map(|client| client.name().to_string())
            .collect();

        if installed_clients.is_empty() {
            println!(
                "\n{} No MCP clients found. Please install Claude Desktop or another MCP client first.",
                "⚠".yellow()
            );
            return Ok(vec![]);
        }

        if installed_clients.len() == 1 {
            let client_name = &installed_clients[0];
            let confirm = Confirm::new()
                .with_prompt(format!("Install to {}?", client_name.cyan()))
                .default(true)
                .interact()?;

            if confirm {
                Ok(vec![client_name.clone()])
            } else {
                Ok(vec![])
            }
        } else {
            println!("\n{}", "Select MCP clients to install to:".blue());
            let selections = dialoguer::MultiSelect::new()
                .items(&installed_clients)
                .defaults(&vec![true; installed_clients.len()])
                .interact()?;

            Ok(selections
                .into_iter()
                .map(|i| installed_clients[i].clone())
                .collect())
        }
    }

    fn prompt_configuration(&self, server: &dyn McpServer) -> Result<HashMap<String, String>> {
        let metadata = server.metadata();
        let mut config = HashMap::new();

        let all_fields: Vec<_> = metadata
            .required_config
            .iter()
            .chain(metadata.optional_config.iter())
            .collect();

        if all_fields.is_empty() {
            if self.verbose {
                eprintln!("{} No configuration required for this server", "ℹ".blue());
            }
            return Ok(config);
        }

        println!("\n{}", "Configuration:".blue().bold());

        for field in all_fields {
            let is_required = metadata
                .required_config
                .iter()
                .any(|f| f.name == field.name);
            let prompt = if let Some(desc) = &field.description {
                if is_required {
                    desc.clone()
                } else {
                    format!("{desc} (optional)")
                }
            } else if is_required {
                field.name.clone()
            } else {
                format!("{} (optional)", field.name)
            };

            let value = match field.field_type {
                ConfigFieldType::String | ConfigFieldType::Path | ConfigFieldType::Url => {
                    let mut input = Input::<String>::new().with_prompt(&prompt);

                    if let Some(default) = &field.default {
                        input = input.default(default.clone());
                    }

                    if !is_required && field.default.is_none() {
                        input = input.allow_empty(true);
                    }

                    let value = input.interact_text()?;

                    if value.is_empty() && !is_required {
                        continue;
                    }

                    value
                }
                ConfigFieldType::Number => {
                    let input = Input::<String>::new()
                        .with_prompt(&prompt)
                        .allow_empty(!is_required)
                        .interact_text()?;

                    if input.is_empty() && !is_required {
                        continue;
                    }

                    let _: f64 = input
                        .parse()
                        .context(format!("Invalid number for {}", field.name))?;
                    input
                }
                ConfigFieldType::Boolean => {
                    let default = field
                        .default
                        .as_ref()
                        .and_then(|v| v.parse::<bool>().ok())
                        .unwrap_or(false);

                    let value = Confirm::new()
                        .with_prompt(&prompt)
                        .default(default)
                        .interact()?;

                    value.to_string()
                }
            };

            config.insert(field.name.clone(), value);
        }

        // Validate the configuration
        server.validate_config(&config)?;

        Ok(config)
    }

    fn install_to_client(
        &self,
        client_name: &str,
        server_name: &str,
        config: &HashMap<String, String>,
    ) -> Result<()> {
        let client = self
            .client_registry
            .get_by_name(client_name)
            .ok_or_else(|| anyhow!("Client {} not found", client_name))?;

        println!("{} Installing to {}...", "→".green(), client_name.cyan());

        let server_config = ServerConfig {
            command: "npx".to_string(), // This will be properly set by the server
            args: vec![
                "--yes".to_string(),
                server_name.to_string(),
                "--stdio".to_string(),
            ],
            env: config.clone(),
        };

        client.add_server(server_name, server_config)?;

        println!("  {} Installed to {}", "✓".green(), client_name);

        Ok(())
    }
}
