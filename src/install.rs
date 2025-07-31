//! MCP server installation command implementation.
//!
//! This module contains the main installation logic for MCP servers. It handles
//! dependency checking, security validation, client detection, server configuration,
//! and the complete installation workflow.
//!
//! # Features
//!
//! - **Multi-Server Support**: NPM packages, Docker images, GitHub repos, binaries
//! - **Multi-Client Integration**: Installs to multiple MCP clients simultaneously
//! - **Dependency Management**: Automatic checking and optional installation
//! - **Security Validation**: Validates server sources and warns about risks
//! - **Interactive Configuration**: Guides users through server setup
//! - **Batch Installation**: Install multiple servers from a file
//! - **Dry Run Mode**: Preview changes without making them
//!
//! # Example
//!
//! ```rust,no_run
//! use mcp_helper::install::InstallCommand;
//!
//! // Create installer with verbose output
//! let mut installer = InstallCommand::new(true);
//!
//! // Configure installer options
//! installer = installer
//!     .with_auto_install_deps(true)
//!     .with_dry_run(false);
//!
//! // Install a server
//! installer.execute("@modelcontextprotocol/server-filesystem")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use colored::Colorize;
use dialoguer::{Confirm, Input};
use std::collections::HashMap;
use std::fs;

use crate::cache::CacheManager;
use crate::client::{detect_clients, ClientRegistry, ServerConfig};
use crate::config::ConfigManager;
use crate::deps::{Dependency, DependencyInstaller, DependencyStatus};
use crate::error::{McpError, Result};
use crate::logging;
use crate::security::SecurityValidator;
use crate::server::{
    detect_server_type, ConfigField, ConfigFieldType, McpServer, ServerSuggestions, ServerType,
};

/// Main installation command for MCP servers.
///
/// The InstallCommand handles the complete workflow of installing MCP servers:
/// dependency checking, security validation, client detection, configuration,
/// and server installation across multiple MCP clients.
pub struct InstallCommand {
    /// Registry of available MCP clients
    client_registry: ClientRegistry,
    /// Configuration manager for atomic updates and rollback
    config_manager: ConfigManager,
    /// Security validator for server source validation
    security_validator: SecurityValidator,
    /// Cache manager for dependency and metadata caching
    cache_manager: CacheManager,
    /// Whether to show verbose output
    verbose: bool,
    /// Whether to automatically install missing dependencies
    auto_install_deps: bool,
    /// Whether to perform a dry run (no actual changes)
    dry_run: bool,
    /// Server suggestion engine for alternatives
    suggestions: ServerSuggestions,
    /// Configuration overrides from command line (key=value pairs)
    config_overrides: HashMap<String, String>,
}

impl InstallCommand {
    /// Create a new installation command with the specified verbosity.
    ///
    /// This constructor automatically detects and registers all available MCP clients,
    /// initializes the configuration manager, and sets up default options.
    ///
    /// # Arguments
    /// * `verbose` - Whether to enable verbose output during installation
    ///
    /// # Returns
    /// A new InstallCommand ready for configuration and execution
    pub fn new(verbose: bool) -> Self {
        // Create an empty registry - clients will be loaded on demand
        let client_registry = ClientRegistry::new();

        Self {
            client_registry,
            config_manager: ConfigManager::new().expect("Failed to create config manager"),
            security_validator: SecurityValidator::new(),
            cache_manager: CacheManager::new().unwrap_or_else(|_| CacheManager::default()),
            verbose,
            auto_install_deps: false,
            dry_run: false,
            suggestions: ServerSuggestions::new(),
            config_overrides: HashMap::new(),
        }
    }

    /// Enable or disable automatic dependency installation.
    ///
    /// When enabled, the installer will attempt to automatically install
    /// missing dependencies (Node.js, Docker, Python, etc.) using the
    /// system package manager.
    ///
    /// # Arguments
    /// * `auto_install` - Whether to automatically install missing dependencies
    pub fn with_auto_install_deps(mut self, auto_install: bool) -> Self {
        self.auto_install_deps = auto_install;
        self
    }

    /// Enable or disable dry run mode.
    ///
    /// In dry run mode, the installer will show what would be done
    /// without making any actual changes to the system or configuration files.
    ///
    /// # Arguments
    /// * `dry_run` - Whether to perform a dry run (no actual changes)
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set configuration overrides from command-line arguments.
    ///
    /// Configuration overrides allow non-interactive installation by
    /// providing server configuration values as key=value pairs.
    ///
    /// # Arguments
    /// * `config_args` - Vector of strings in "key=value" format
    ///
    /// # Example
    /// ```rust,no_run
    /// use mcp_helper::install::InstallCommand;
    ///
    /// let installer = InstallCommand::new(false)
    ///     .with_config_overrides(vec![
    ///         "allowedDirectories=/home/user/docs".to_string(),
    ///         "allowedFileTypes=.md,.txt".to_string(),
    ///     ]);
    /// ```
    pub fn with_config_overrides(mut self, config_args: Vec<String>) -> Self {
        self.config_overrides = Self::parse_config_args(&config_args);
        self
    }

    fn parse_config_args(config_args: &[String]) -> HashMap<String, String> {
        let mut config = HashMap::new();

        for arg in config_args {
            if let Some((key, value)) = arg.split_once('=') {
                config.insert(key.trim().to_string(), value.trim().to_string());
            } else {
                eprintln!(
                    "{} Invalid config format: '{}'. Expected key=value",
                    "⚠".yellow(),
                    arg
                );
            }
        }

        config
    }

    /// Execute the installation of a single MCP server.
    ///
    /// This is the main entry point for installing an MCP server. It performs
    /// the complete installation workflow including dependency checking,
    /// security validation, client selection, configuration, and installation.
    ///
    /// # Arguments
    /// * `server_name` - Name or specification of the server to install
    ///   - NPM packages: `@org/package-name` or `package-name`
    ///   - Docker images: `docker:image:tag`
    ///   - GitHub repos: `user/repo` or `https://github.com/user/repo`
    ///   - Binaries: `https://example.com/path/to/binary`
    ///
    /// # Returns
    /// `Ok(())` if installation succeeds, or an error describing what went wrong
    ///
    /// # Example
    /// ```rust,no_run
    /// use mcp_helper::install::InstallCommand;
    ///
    /// let mut installer = InstallCommand::new(true);
    /// installer.execute("@modelcontextprotocol/server-filesystem")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn execute(&mut self, server_name: &str) -> Result<()> {
        if self.verbose {
            eprintln!("{} Detecting server type for: {}", "ℹ".blue(), server_name);
        }

        // Validate server source security
        self.validate_server_security(server_name)?;

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

        // Log successful server installation
        let server_type_name = match server_type {
            ServerType::Npm { .. } => "npm",
            ServerType::Binary { .. } => "binary",
            ServerType::Python { .. } => "python",
            ServerType::Docker { .. } => "docker",
        };
        logging::log_server_installation(server_name, server_type_name, true);

        println!(
            "\n{} Successfully installed {} to {} client(s)",
            "✓".green().bold(),
            server_name.cyan(),
            clients.len()
        );

        Ok(())
    }

    /// Execute batch installation of multiple MCP servers from a file.
    ///
    /// Reads a file containing server specifications (one per line) and installs
    /// each server sequentially. Empty lines and lines starting with '#' are ignored.
    /// If any server fails to install, the process continues with the remaining servers.
    ///
    /// # Arguments
    /// * `batch_file` - Path to the batch file containing server specifications
    ///
    /// # Batch File Format
    /// ```text
    /// # MCP servers to install
    /// @modelcontextprotocol/server-filesystem
    /// @anthropic/mcp-server-slack
    /// docker:postgres:13
    /// user/custom-mcp-server
    /// ```
    ///
    /// # Returns
    /// `Ok(())` if the batch file was processed, regardless of individual server success/failure
    ///
    /// # Example
    /// ```rust,no_run
    /// use mcp_helper::install::InstallCommand;
    ///
    /// let mut installer = InstallCommand::new(true);
    /// installer.execute_batch("servers.txt")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn execute_batch(&mut self, batch_file: &str) -> Result<()> {
        let batch_content = fs::read_to_string(batch_file).map_err(|e| {
            McpError::Other(anyhow::anyhow!(
                "Failed to read batch file '{}': {}",
                batch_file,
                e
            ))
        })?;

        let batch_config = Self::parse_batch_file(&batch_content)?;

        if batch_config.is_empty() {
            return Err(McpError::Other(anyhow::anyhow!(
                "No servers found in batch file"
            )));
        }

        println!(
            "{} Found {} server(s) to install",
            "ℹ".blue(),
            batch_config.len()
        );

        let mut success_count = 0;
        let mut failure_count = 0;
        let mut failures = Vec::new();

        for (server_name, server_config) in batch_config {
            println!("\n{} Installing {}", "→".green(), server_name.cyan());

            // Set config overrides for this server
            self.config_overrides = server_config;

            match self.execute(&server_name) {
                Ok(()) => {
                    success_count += 1;
                    println!("  {} Successfully installed {}", "✓".green(), server_name);
                }
                Err(e) => {
                    failure_count += 1;
                    failures.push((server_name.clone(), e.to_string()));
                    eprintln!("  {} Failed to install {}: {}", "✗".red(), server_name, e);
                }
            }
        }

        println!("\n{} Batch installation complete:", "📊".blue());
        println!("  {} {} successful", "✓".green(), success_count);

        if failure_count > 0 {
            println!("  {} {} failed", "✗".red(), failure_count);
            println!("\n{} Failed installations:", "❌".red());
            for (server, error) in failures {
                println!("  • {}: {}", server.cyan(), error);
            }
            return Err(McpError::Other(anyhow::anyhow!(
                "{} out of {} installations failed",
                failure_count,
                success_count + failure_count
            )));
        }

        Ok(())
    }

    fn parse_batch_file(content: &str) -> Result<HashMap<String, HashMap<String, String>>> {
        let mut servers = HashMap::new();
        let mut current_server: Option<String> = None;
        let mut current_config = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check if this is a server declaration
            if line.starts_with('[') && line.ends_with(']') {
                // Save previous server if exists
                if let Some(server_name) = current_server.take() {
                    servers.insert(server_name, current_config.clone());
                    current_config.clear();
                }

                // Start new server
                current_server = Some(line[1..line.len() - 1].to_string());
                continue;
            }

            // Parse key=value configuration
            if let Some((key, value)) = line.split_once('=') {
                current_config.insert(key.trim().to_string(), value.trim().to_string());
            } else {
                return Err(McpError::Other(anyhow::anyhow!(
                    "Invalid line in batch file: '{}'. Expected key=value or [server-name]",
                    line
                )));
            }
        }

        // Save the last server
        if let Some(server_name) = current_server {
            servers.insert(server_name, current_config);
        }

        Ok(servers)
    }

    fn validate_server_security(&self, server_name: &str) -> Result<()> {
        let validation =
            if server_name.starts_with("http://") || server_name.starts_with("https://") {
                // Direct URL
                self.security_validator.validate_url(server_name)?
            } else if server_name.starts_with("docker:") {
                // Docker image
                let image_name = server_name.strip_prefix("docker:").unwrap_or(server_name);
                self.security_validator.validate_docker_image(image_name)?
            } else if server_name.contains('/') && !server_name.starts_with('@') {
                // Likely a GitHub repo or similar
                let url = if server_name.starts_with("github.com/") {
                    format!("https://{server_name}")
                } else {
                    format!("https://github.com/{server_name}")
                };
                self.security_validator.validate_url(&url)?
            } else {
                // NPM package
                self.security_validator.validate_npm_package(server_name)?
            };

        // Log security check
        tracing::info!(
            server = server_name,
            is_trusted = validation.is_trusted,
            is_https = validation.is_https,
            domain = validation.domain.as_deref().unwrap_or("unknown"),
            "Security validation completed"
        );

        // Handle warnings
        if !validation.warnings.is_empty() {
            println!(
                "{} {}",
                "⚠".yellow(),
                "Security warnings detected:".yellow()
            );
            for warning in &validation.warnings {
                println!("  {} {}", "•".yellow(), warning);
            }

            // Block if validation says we should
            if validation.should_block() {
                return Err(McpError::Other(anyhow::anyhow!(
                    "Installation blocked due to security concerns. Use --force to override (if available)."
                )));
            }

            // Prompt user for non-blocking warnings
            if !validation.is_safe() && !self.dry_run {
                println!();
                let proceed = Confirm::new()
                    .with_prompt("Do you want to proceed despite these warnings?")
                    .default(false)
                    .interact()
                    .map_err(|e| {
                        McpError::Other(anyhow::anyhow!("Failed to read user input: {}", e))
                    })?;

                if !proceed {
                    return Err(McpError::Other(anyhow::anyhow!(
                        "Installation cancelled by user due to security warnings."
                    )));
                }
            }
        } else if self.verbose {
            println!("{} Security validation passed", "✓".green());
        }

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

        // Cache the result for future use
        if let Err(e) = self
            .cache_manager
            .cache_dependency_status(check.dependency.clone(), check.status.clone())
        {
            if self.verbose {
                eprintln!("{} Failed to cache dependency status: {}", "⚠".yellow(), e);
            }
        }

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

    fn ensure_clients_loaded(&mut self) {
        if self.client_registry.clients.is_empty() {
            if self.verbose {
                eprintln!("{} Loading MCP clients...", "ℹ".blue());
            }

            // Load clients on demand
            for client in detect_clients() {
                self.client_registry.register(client);
            }
        }
    }

    fn select_clients(&mut self) -> Result<Vec<String>> {
        // Ensure clients are loaded
        self.ensure_clients_loaded();
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

        // Start with config overrides from command line
        config.extend(self.config_overrides.clone());

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

        // Check if we're in non-interactive mode (have config overrides)
        let is_non_interactive = !self.config_overrides.is_empty();

        if is_non_interactive {
            if self.verbose {
                eprintln!(
                    "{} Using non-interactive mode with provided configuration",
                    "ℹ".blue()
                );
            }
        } else {
            println!("\n{}", "Configuration:".blue().bold());
        }

        for field in all_fields {
            // Skip prompting if we already have this field from overrides
            if config.contains_key(&field.name) {
                if self.verbose {
                    eprintln!(
                        "  {} Using override for {}: {}",
                        "→".green(),
                        field.name,
                        config[&field.name]
                    );
                }
                continue;
            }

            let is_required = metadata
                .required_config
                .iter()
                .any(|f| f.name == field.name);

            // In non-interactive mode, skip optional fields but fail on required ones
            if is_non_interactive {
                if is_required {
                    return Err(McpError::Other(anyhow::anyhow!(
                        "Required configuration field '{}' not provided via --config",
                        field.name
                    )));
                }
                continue;
            }

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

        // Validate the configuration using ConfigManager
        if let Err(validation_errors) = self.config_manager.validate_config(server, &config) {
            for error in &validation_errors {
                eprintln!("  {} {}", "✗".red(), error);
            }
            return Err(McpError::Other(anyhow::anyhow!(
                "Configuration validation failed with {} error(s)",
                validation_errors.len()
            )));
        }

        Ok(config)
    }

    fn install_to_client(
        &mut self,
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

        // Use ConfigManager to apply configuration with automatic backup
        match self
            .config_manager
            .apply_config(client, server_name, server_config)
        {
            Ok(snapshot) => {
                logging::log_config_change(client_name, server_name, "add");
                println!("  {} Installed to {}", "✓".green(), client_name);
                if self.verbose {
                    println!(
                        "  {} Configuration snapshot saved: {}",
                        "ℹ".blue(),
                        snapshot.timestamp.format("%Y-%m-%d %H:%M:%S")
                    );
                }
            }
            Err(e) => {
                eprintln!("  {} Installation failed: {}", "✗".red(), e);
                return Err(e.into());
            }
        }

        Ok(())
    }
}
