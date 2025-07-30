use colored::Colorize;
use dialoguer::{Confirm, Input};
use std::collections::HashMap;

use crate::client::{detect_clients, ClientRegistry, ServerConfig};
use crate::deps::{Dependency, DependencyInstaller, DependencyStatus};
use crate::error::{McpError, Result};
use crate::server::{
    detect_server_type, ConfigField, ConfigFieldType, McpServer, ServerSuggestions, ServerType,
};

pub struct InstallCommand {
    client_registry: ClientRegistry,
    verbose: bool,
    auto_install_deps: bool,
    dry_run: bool,
    suggestions: ServerSuggestions,
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
            auto_install_deps: false,
            dry_run: false,
            suggestions: ServerSuggestions::new(),
        }
    }

    pub fn with_auto_install_deps(mut self, auto_install: bool) -> Self {
        self.auto_install_deps = auto_install;
        self
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn execute(&mut self, server_name: &str) -> Result<()> {
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
            return Err(McpError::Other(anyhow::anyhow!(
                "No MCP clients selected for installation"
            )));
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
            ServerType::Binary { url, checksum } => {
                use crate::server::binary::BinaryServer;
                Ok(Box::new(BinaryServer::new(url, checksum.clone())))
            }
            ServerType::Python { package, version } => {
                use crate::server::python::PythonServer;
                let package_spec = if let Some(v) = version {
                    format!("{package}=={v}")
                } else {
                    package.clone()
                };
                Ok(Box::new(PythonServer::new(&package_spec)?))
            }
            ServerType::Docker { image, tag } => {
                use crate::server::docker::DockerServer;
                let docker_spec = if let Some(t) = tag {
                    format!("{image}:{t}")
                } else {
                    image.clone()
                };
                Ok(Box::new(DockerServer::new(&docker_spec)?))
            }
        }
    }

    pub fn get_dependency_name(dependency: &Dependency) -> &'static str {
        match dependency {
            Dependency::NodeJs { .. } => "Node.js",
            Dependency::Python { .. } => "Python",
            Dependency::Docker { .. } => "Docker",
            Dependency::Git => "Git",
        }
    }

    pub fn handle_installed_dependency(dep_name: &str, version: &Option<String>) -> Result<()> {
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

    pub fn handle_missing_dependency(
        dep_name: &str,
        check: &crate::deps::DependencyCheck,
    ) -> Result<()> {
        println!("  {} {} is not installed", "✗".red(), dep_name);

        if let Some(instructions) = &check.install_instructions {
            let required_version = match &check.dependency {
                Dependency::NodeJs { min_version } => min_version.clone(),
                Dependency::Python { min_version } => min_version.clone(),
                _ => None,
            };
            return Err(McpError::missing_dependency(
                dep_name,
                required_version,
                instructions.clone(),
            ));
        }
        Err(McpError::Other(anyhow::anyhow!(
            "Required dependency {} is not installed",
            dep_name
        )))
    }

    fn check_dependencies(&mut self, server: &dyn McpServer) -> Result<()> {
        println!("{} Checking dependencies...", "🔍".blue());

        let dependency = server.dependency();
        let check = dependency.check()?;

        let dep_name = Self::get_dependency_name(&check.dependency);

        match &check.status {
            DependencyStatus::Installed { version } => {
                Self::handle_installed_dependency(dep_name, version)
            }
            DependencyStatus::Missing => {
                if self.auto_install_deps {
                    self.attempt_auto_install(dep_name, &check)
                } else {
                    Self::handle_missing_dependency(dep_name, &check)
                }
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

                if self.auto_install_deps {
                    self.attempt_auto_install(dep_name, &check)
                } else if let Some(instructions) = &check.install_instructions {
                    Err(McpError::version_mismatch(
                        dep_name,
                        installed,
                        required,
                        instructions.clone(),
                    ))
                } else {
                    Err(McpError::Other(anyhow::anyhow!(
                        "Dependency {} version mismatch",
                        dep_name
                    )))
                }
            }
            DependencyStatus::ConfigurationRequired { issue, solution } => {
                println!(
                    "  {} {} configuration issue: {}",
                    "⚠".yellow(),
                    dep_name,
                    issue
                );
                println!("  {} Solution: {}", "💡".blue(), solution);

                Err(McpError::Other(anyhow::anyhow!(
                    "Dependency {} requires configuration: {}. {}",
                    dep_name,
                    issue,
                    solution
                )))
            }
        }
    }

    fn attempt_auto_install(
        &mut self,
        dep_name: &str,
        check: &crate::deps::DependencyCheck,
    ) -> Result<()> {
        println!(
            "  {} Attempting to auto-install {}...",
            "🚀".blue(),
            dep_name
        );

        let mut installer = DependencyInstaller::new();
        if self.dry_run {
            installer = installer.with_dry_run();
        }
        if self.auto_install_deps {
            installer = installer.with_auto_confirm();
        }

        // Show elevation warning if needed
        if let Some(warning) = installer.get_elevation_warning(&check.dependency) {
            println!("  {} {}", "⚠".yellow(), warning);
        }

        match installer.install_dependency(check) {
            Ok(true) => {
                println!("  {} Successfully installed {}", "✅".green(), dep_name);
                Ok(())
            }
            Ok(false) => {
                println!("  {} Could not auto-install {}", "⚠".yellow(), dep_name);
                self.show_suggestions_for_dependency(dep_name, &check.dependency)?;
                Self::handle_missing_dependency(dep_name, check)
            }
            Err(e) => {
                println!("  {} Auto-installation failed: {}", "❌".red(), e);
                self.show_suggestions_for_dependency(dep_name, &check.dependency)?;
                Self::handle_missing_dependency(dep_name, check)
            }
        }
    }

    fn show_suggestions_for_dependency(
        &mut self,
        _dep_name: &str,
        failed_dependency: &Dependency,
    ) -> Result<()> {
        println!("\n{} Looking for alternative servers...", "💡".blue());

        let alternatives = self
            .suggestions
            .suggest_alternatives("unknown-server", Some(failed_dependency));

        if alternatives.is_empty() {
            println!("  {} No alternative servers found", "ℹ".blue());
            return Ok(());
        }

        println!(
            "  {} Found {} alternative server(s):",
            "✨".green(),
            alternatives.len()
        );

        for (i, suggestion) in alternatives.iter().enumerate() {
            println!(
                "    {}. {} - {}",
                i + 1,
                suggestion.server.name.cyan(),
                suggestion.server.description
            );
            println!("       {} Reason: {}", "→".blue(), suggestion.reason);

            let feasibility = self.suggestions.check_suggestion_feasibility(suggestion);
            println!("       {} Status: {}", "🔍".blue(), feasibility);

            if suggestion.server.verified {
                println!("       {} Verified server", "✅".green());
            }

            println!(
                "       {} Install: mcp install {}",
                "📦".blue(),
                suggestion.server.package_name
            );
            println!();
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

    pub fn build_field_prompt(field: &ConfigField, is_required: bool) -> String {
        match (&field.description, is_required) {
            (Some(desc), true) => desc.clone(),
            (Some(desc), false) => format!("{desc} (optional)"),
            (None, true) => field.name.clone(),
            (None, false) => format!("{} (optional)", field.name),
        }
    }

    fn prompt_string_field(
        &self,
        field: &ConfigField,
        prompt: &str,
        is_required: bool,
    ) -> Result<Option<String>> {
        let mut input = Input::<String>::new().with_prompt(prompt);

        if let Some(default) = &field.default {
            input = input.default(default.clone());
        }

        if !is_required && field.default.is_none() {
            input = input.allow_empty(true);
        }

        let value = input.interact_text()?;

        if value.is_empty() && !is_required {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    fn prompt_number_field(
        &self,
        field: &ConfigField,
        prompt: &str,
        is_required: bool,
        server_name: &str,
    ) -> Result<Option<String>> {
        let input = Input::<String>::new()
            .with_prompt(prompt)
            .allow_empty(!is_required)
            .interact_text()?;

        if input.is_empty() && !is_required {
            return Ok(None);
        }

        let _: f64 = input.parse().map_err(|_| {
            McpError::configuration_required(
                server_name,
                vec![field.name.clone()],
                vec![(field.name.clone(), "Must be a valid number".to_string())],
            )
        })?;
        Ok(Some(input))
    }

    fn prompt_boolean_field(&self, field: &ConfigField, prompt: &str) -> Result<String> {
        let default = field
            .default
            .as_ref()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        let value = Confirm::new()
            .with_prompt(prompt)
            .default(default)
            .interact()?;

        Ok(value.to_string())
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
            let prompt = Self::build_field_prompt(field, is_required);

            let value = match field.field_type {
                ConfigFieldType::String | ConfigFieldType::Path | ConfigFieldType::Url => {
                    match self.prompt_string_field(field, &prompt, is_required)? {
                        Some(v) => v,
                        None => continue,
                    }
                }
                ConfigFieldType::Number => {
                    match self.prompt_number_field(field, &prompt, is_required, &metadata.name)? {
                        Some(v) => v,
                        None => continue,
                    }
                }
                ConfigFieldType::Boolean => self.prompt_boolean_field(field, &prompt)?,
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
            .ok_or_else(|| {
                let available_clients = self
                    .client_registry
                    .detect_installed()
                    .into_iter()
                    .map(|c| c.name().to_string())
                    .collect();
                McpError::client_not_found(
                    client_name,
                    available_clients,
                    "Please check the client name and try again",
                )
            })?;

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
