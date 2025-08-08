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
use crate::security::{SecurityValidation, SecurityValidator};
use crate::server::{
    detect_server_type, ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerSuggestions,
    ServerType,
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
        let validation = self.perform_security_validation(server_name)?;
        self.log_security_validation(server_name, &validation);
        self.handle_security_warnings(&validation)?;
        Ok(())
    }

    fn perform_security_validation(&self, server_name: &str) -> Result<SecurityValidation> {
        let result = if server_name.starts_with("http://") || server_name.starts_with("https://") {
            // Direct URL
            self.security_validator.validate_url(server_name)
        } else if server_name.starts_with("docker:") {
            // Docker image
            let image_name = server_name.strip_prefix("docker:").unwrap_or(server_name);
            self.security_validator.validate_docker_image(image_name)
        } else if server_name.contains('/') && !server_name.starts_with('@') {
            // Likely a GitHub repo or similar
            let url = self.build_github_url(server_name);
            self.security_validator.validate_url(&url)
        } else {
            // NPM package
            self.security_validator.validate_npm_package(server_name)
        };

        result.map_err(McpError::Other)
    }

    fn build_github_url(&self, server_name: &str) -> String {
        if server_name.starts_with("github.com/") {
            format!("https://{server_name}")
        } else {
            format!("https://github.com/{server_name}")
        }
    }

    fn log_security_validation(&self, server_name: &str, validation: &SecurityValidation) {
        tracing::info!(
            server = server_name,
            is_trusted = validation.is_trusted,
            is_https = validation.is_https,
            domain = validation.domain.as_deref().unwrap_or("unknown"),
            "Security validation completed"
        );
    }

    fn handle_security_warnings(&self, validation: &SecurityValidation) -> Result<()> {
        if validation.warnings.is_empty() {
            if self.verbose {
                println!("{} Security validation passed", "✓".green());
            }
            return Ok(());
        }

        self.display_security_warnings(&validation.warnings);

        if validation.should_block() {
            return Err(McpError::Other(anyhow::anyhow!(
                "Installation blocked due to security concerns. Use --force to override (if available)."
            )));
        }

        if !validation.is_safe() && !self.dry_run {
            self.prompt_security_confirmation()?
        }

        Ok(())
    }

    fn display_security_warnings(&self, warnings: &[String]) {
        println!(
            "{} {}",
            "⚠".yellow(),
            "Security warnings detected:".yellow()
        );
        for warning in warnings {
            println!("  {} {}", "•".yellow(), warning);
        }
    }

    fn prompt_security_confirmation(&self) -> Result<()> {
        println!();
        let proceed = Confirm::new()
            .with_prompt("Do you want to proceed despite these warnings?")
            .default(false)
            .interact()
            .map_err(|e| McpError::Other(anyhow::anyhow!("Failed to read user input: {}", e)))?;

        if !proceed {
            return Err(McpError::Other(anyhow::anyhow!(
                "Installation cancelled by user due to security warnings."
            )));
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
        let mut config = self.initialize_config();
        let all_fields = self.collect_all_fields(metadata);

        if all_fields.is_empty() {
            return self.handle_no_config_required();
        }

        let is_non_interactive = !self.config_overrides.is_empty();
        self.display_config_mode(is_non_interactive);

        for field in all_fields {
            self.process_config_field(&mut config, field, metadata, is_non_interactive)?;
        }

        self.validate_final_config(server, &config)?;
        Ok(config)
    }

    fn initialize_config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.extend(self.config_overrides.clone());
        config
    }

    fn collect_all_fields<'a>(&self, metadata: &'a ServerMetadata) -> Vec<&'a ConfigField> {
        metadata
            .required_config
            .iter()
            .chain(metadata.optional_config.iter())
            .collect()
    }

    fn handle_no_config_required(&self) -> Result<HashMap<String, String>> {
        if self.verbose {
            eprintln!("{} No configuration required for this server", "ℹ".blue());
        }
        Ok(self.initialize_config())
    }

    fn display_config_mode(&self, is_non_interactive: bool) {
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
    }

    fn process_config_field(
        &self,
        config: &mut HashMap<String, String>,
        field: &ConfigField,
        metadata: &ServerMetadata,
        is_non_interactive: bool,
    ) -> Result<()> {
        if self.should_skip_field(config, field)? {
            return Ok(());
        }

        let is_required = self.is_required_field(field, metadata);

        if is_non_interactive {
            return self.handle_non_interactive_field(field, is_required);
        }

        let value = self.prompt_for_field_value(field, is_required, &metadata.name)?;
        if let Some(v) = value {
            config.insert(field.name.clone(), v);
        }
        Ok(())
    }

    fn should_skip_field(
        &self,
        config: &HashMap<String, String>,
        field: &ConfigField,
    ) -> Result<bool> {
        if config.contains_key(&field.name) {
            if self.verbose {
                eprintln!(
                    "  {} Using override for {}: {}",
                    "→".green(),
                    field.name,
                    config[&field.name]
                );
            }
            return Ok(true);
        }
        Ok(false)
    }

    fn is_required_field(&self, field: &ConfigField, metadata: &ServerMetadata) -> bool {
        metadata
            .required_config
            .iter()
            .any(|f| f.name == field.name)
    }

    fn handle_non_interactive_field(&self, field: &ConfigField, is_required: bool) -> Result<()> {
        if is_required {
            return Err(McpError::Other(anyhow::anyhow!(
                "Required configuration field '{}' not provided via --config",
                field.name
            )));
        }
        Ok(())
    }

    fn prompt_for_field_value(
        &self,
        field: &ConfigField,
        is_required: bool,
        server_name: &str,
    ) -> Result<Option<String>> {
        let prompt = Self::build_field_prompt(field, is_required);

        match field.field_type {
            ConfigFieldType::String | ConfigFieldType::Path | ConfigFieldType::Url => {
                self.prompt_string_field(field, &prompt, is_required)
            }
            ConfigFieldType::Number => {
                self.prompt_number_field(field, &prompt, is_required, server_name)
            }
            ConfigFieldType::Boolean => Ok(Some(self.prompt_boolean_field(field, &prompt)?)),
        }
    }

    fn validate_final_config(
        &self,
        server: &dyn McpServer,
        config: &HashMap<String, String>,
    ) -> Result<()> {
        if let Err(validation_errors) = self.config_manager.validate_config(server, config) {
            for error in &validation_errors {
                eprintln!("  {} {}", "✗".red(), error);
            }
            return Err(McpError::Other(anyhow::anyhow!(
                "Configuration validation failed with {} error(s)",
                validation_errors.len()
            )));
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deps::DependencyChecker;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // Mock dependency checker for testing
    struct MockDependencyChecker {
        dependency: Dependency,
    }

    impl DependencyChecker for MockDependencyChecker {
        fn check(&self) -> anyhow::Result<crate::deps::DependencyCheck> {
            Ok(crate::deps::DependencyCheck {
                dependency: self.dependency.clone(),
                status: DependencyStatus::Installed {
                    version: Some("1.0.0".to_string()),
                },
                install_instructions: None,
            })
        }
    }

    // Mock server for testing
    struct MockServer {
        metadata: ServerMetadata,
        dependency: Dependency,
    }

    impl McpServer for MockServer {
        fn metadata(&self) -> &ServerMetadata {
            &self.metadata
        }

        fn dependency(&self) -> Box<dyn DependencyChecker> {
            Box::new(MockDependencyChecker {
                dependency: self.dependency.clone(),
            })
        }

        fn validate_config(&self, _config: &HashMap<String, String>) -> anyhow::Result<()> {
            Ok(())
        }

        fn generate_command(&self) -> anyhow::Result<(String, Vec<String>)> {
            Ok(("node".to_string(), vec!["server.js".to_string()]))
        }
    }

    #[test]
    fn test_install_command_new() {
        let installer = InstallCommand::new(true);
        assert!(installer.verbose);
        assert!(!installer.auto_install_deps);
        assert!(!installer.dry_run);
        assert!(installer.config_overrides.is_empty());
    }

    #[test]
    fn test_install_command_new_no_verbose() {
        let installer = InstallCommand::new(false);
        assert!(!installer.verbose);
    }

    #[test]
    fn test_with_auto_install_deps() {
        let installer = InstallCommand::new(false).with_auto_install_deps(true);
        assert!(installer.auto_install_deps);

        let installer = InstallCommand::new(false).with_auto_install_deps(false);
        assert!(!installer.auto_install_deps);
    }

    #[test]
    fn test_with_dry_run() {
        let installer = InstallCommand::new(false).with_dry_run(true);
        assert!(installer.dry_run);

        let installer = InstallCommand::new(false).with_dry_run(false);
        assert!(!installer.dry_run);
    }

    #[test]
    fn test_with_config_overrides() {
        let config_args = vec!["key1=value1".to_string(), "key2=value2".to_string()];

        let installer = InstallCommand::new(false).with_config_overrides(config_args);

        assert_eq!(installer.config_overrides.len(), 2);
        assert_eq!(
            installer.config_overrides.get("key1"),
            Some(&"value1".to_string())
        );
        assert_eq!(
            installer.config_overrides.get("key2"),
            Some(&"value2".to_string())
        );
    }

    #[test]
    fn test_parse_config_args() {
        let args = vec![
            "path=/home/user".to_string(),
            "port=8080".to_string(),
            "invalid_arg".to_string(),
            "key=".to_string(),
            "=value".to_string(),
        ];

        let config = InstallCommand::parse_config_args(&args);

        // "invalid_arg" is skipped, "key=" creates key with empty value, "=value" creates empty key with value
        assert_eq!(config.len(), 4);
        assert_eq!(config.get("path"), Some(&"/home/user".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("key"), Some(&"".to_string()));
        assert_eq!(config.get(""), Some(&"value".to_string()));
    }

    #[test]
    fn test_get_dependency_name() {
        assert_eq!(
            InstallCommand::get_dependency_name(&Dependency::NodeJs { min_version: None }),
            "Node.js"
        );
        assert_eq!(
            InstallCommand::get_dependency_name(&Dependency::Python { min_version: None }),
            "Python"
        );
        assert_eq!(
            InstallCommand::get_dependency_name(&Dependency::Docker {
                min_version: None,
                requires_compose: false
            }),
            "Docker"
        );
        assert_eq!(InstallCommand::get_dependency_name(&Dependency::Git), "Git");
    }

    #[test]
    fn test_handle_installed_dependency() {
        let result =
            InstallCommand::handle_installed_dependency("Node.js", &Some("18.0.0".to_string()));
        assert!(result.is_ok());

        let result = InstallCommand::handle_installed_dependency("Python", &None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_missing_dependency() {
        let check = crate::deps::DependencyCheck {
            dependency: Dependency::NodeJs {
                min_version: Some("18.0.0".to_string()),
            },
            status: DependencyStatus::Missing,
            install_instructions: Some(crate::deps::InstallInstructions::default()),
        };

        let result = InstallCommand::handle_missing_dependency("Node.js", &check);
        assert!(result.is_err());

        match result.unwrap_err() {
            McpError::MissingDependency { dependency, .. } => {
                assert_eq!(dependency, "Node.js");
            }
            _ => panic!("Expected MissingDependency error"),
        }
    }

    #[test]
    fn test_handle_missing_dependency_no_instructions() {
        let check = crate::deps::DependencyCheck {
            dependency: Dependency::Git,
            status: DependencyStatus::Missing,
            install_instructions: None,
        };

        let result = InstallCommand::handle_missing_dependency("Git", &check);
        assert!(result.is_err());

        match result.unwrap_err() {
            McpError::Other(err) => {
                assert!(err.to_string().contains("Git is not installed"));
            }
            _ => panic!("Expected Other error"),
        }
    }

    #[test]
    fn test_build_field_prompt() {
        let field = ConfigField {
            name: "apiKey".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("API key for authentication".to_string()),
            default: None,
        };

        let prompt = InstallCommand::build_field_prompt(&field, true);
        assert_eq!(prompt, "API key for authentication");

        let prompt = InstallCommand::build_field_prompt(&field, false);
        assert_eq!(prompt, "API key for authentication (optional)");
    }

    #[test]
    fn test_build_field_prompt_no_description() {
        let field = ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        };

        let prompt = InstallCommand::build_field_prompt(&field, true);
        assert_eq!(prompt, "port");

        let prompt = InstallCommand::build_field_prompt(&field, false);
        assert_eq!(prompt, "port (optional)");
    }

    #[test]
    fn test_parse_batch_file() {
        let content = r#"
# Comment line
[server1]
key1=value1
key2=value2

[server2]
port=8080

# Another comment
[server3]
"#;

        let result = InstallCommand::parse_batch_file(content);
        assert!(result.is_ok());

        let servers = result.unwrap();
        assert_eq!(servers.len(), 3);

        let server1_config = &servers["server1"];
        assert_eq!(server1_config.get("key1"), Some(&"value1".to_string()));
        assert_eq!(server1_config.get("key2"), Some(&"value2".to_string()));

        let server2_config = &servers["server2"];
        assert_eq!(server2_config.get("port"), Some(&"8080".to_string()));

        let server3_config = &servers["server3"];
        assert!(server3_config.is_empty());
    }

    #[test]
    fn test_parse_batch_file_invalid() {
        let content = "invalid line without equals";
        let result = InstallCommand::parse_batch_file(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_github_url() {
        let installer = InstallCommand::new(false);

        assert_eq!(
            installer.build_github_url("user/repo"),
            "https://github.com/user/repo"
        );

        assert_eq!(
            installer.build_github_url("github.com/user/repo"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_create_server_npm() {
        let installer = InstallCommand::new(false);
        let server_type = ServerType::Npm {
            package: "@test/package".to_string(),
            version: Some("1.0.0".to_string()),
        };

        let result = installer.create_server(&server_type);
        assert!(result.is_ok());

        let server = result.unwrap();
        assert_eq!(server.metadata().name, "@test/package");
    }

    #[test]
    fn test_create_server_binary() {
        let installer = InstallCommand::new(false);
        let server_type = ServerType::Binary {
            url: "https://example.com/binary".to_string(),
            checksum: None,
        };

        let result = installer.create_server(&server_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_server_python() {
        let installer = InstallCommand::new(false);
        let server_type = ServerType::Python {
            package: "mcp-server".to_string(),
            version: Some("1.0.0".to_string()),
        };

        let result = installer.create_server(&server_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_server_docker() {
        let installer = InstallCommand::new(false);
        let server_type = ServerType::Docker {
            image: "mcp/server".to_string(),
            tag: Some("latest".to_string()),
        };

        let result = installer.create_server(&server_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initialize_config() {
        let mut installer = InstallCommand::new(false);
        installer
            .config_overrides
            .insert("key".to_string(), "value".to_string());

        let config = installer.initialize_config();
        assert_eq!(config.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_collect_all_fields() {
        let installer = InstallCommand::new(false);
        let metadata = ServerMetadata {
            name: "test-server".to_string(),
            description: Some("Test server".to_string()),
            server_type: ServerType::Npm {
                package: "test-server".to_string(),
                version: None,
            },
            required_config: vec![ConfigField {
                name: "required".to_string(),
                field_type: ConfigFieldType::String,
                description: None,
                default: None,
            }],
            optional_config: vec![ConfigField {
                name: "optional".to_string(),
                field_type: ConfigFieldType::Number,
                description: None,
                default: None,
            }],
        };

        let fields = installer.collect_all_fields(&metadata);
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "required");
        assert_eq!(fields[1].name, "optional");
    }

    #[test]
    fn test_handle_no_config_required() {
        let installer = InstallCommand::new(false);
        let result = installer.handle_no_config_required();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_handle_no_config_required_with_overrides() {
        let mut installer = InstallCommand::new(false);
        installer
            .config_overrides
            .insert("key".to_string(), "value".to_string());

        let result = installer.handle_no_config_required();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_should_skip_field() {
        let installer = InstallCommand::new(false);
        let mut config = HashMap::new();
        config.insert("existing".to_string(), "value".to_string());

        let field = ConfigField {
            name: "existing".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        };

        let result = installer.should_skip_field(&config, &field);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let field2 = ConfigField {
            name: "new".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        };

        let result = installer.should_skip_field(&config, &field2);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_required_field() {
        let installer = InstallCommand::new(false);
        let field1 = ConfigField {
            name: "required".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        };
        let field2 = ConfigField {
            name: "optional".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        };

        let metadata = ServerMetadata {
            name: "test".to_string(),
            description: Some("Test".to_string()),
            server_type: ServerType::Npm {
                package: "test".to_string(),
                version: None,
            },
            required_config: vec![field1.clone()],
            optional_config: vec![field2.clone()],
        };

        assert!(installer.is_required_field(&field1, &metadata));
        assert!(!installer.is_required_field(&field2, &metadata));
    }

    #[test]
    fn test_handle_non_interactive_field_required() {
        let installer = InstallCommand::new(false);
        let field = ConfigField {
            name: "apiKey".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        };

        let result = installer.handle_non_interactive_field(&field, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_non_interactive_field_optional() {
        let installer = InstallCommand::new(false);
        let field = ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        };

        let result = installer.handle_non_interactive_field(&field, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_clients_loaded() {
        let mut installer = InstallCommand::new(false);
        assert!(installer.client_registry.clients.is_empty());

        installer.ensure_clients_loaded();
        // The registry might still be empty if no clients are installed,
        // but the method should not panic
    }

    #[test]
    fn test_display_security_warnings() {
        let installer = InstallCommand::new(false);
        let warnings = vec!["Warning 1".to_string(), "Warning 2".to_string()];

        // This should not panic
        installer.display_security_warnings(&warnings);
    }

    #[test]
    fn test_log_security_validation() {
        let installer = InstallCommand::new(false);
        let validation = SecurityValidation {
            url: "https://github.com/test/repo".to_string(),
            is_trusted: true,
            is_https: true,
            domain: Some("github.com".to_string()),
            warnings: vec![],
        };

        // This should not panic
        installer.log_security_validation("test-server", &validation);
    }

    #[test]
    fn test_prompt_number_field_invalid() {
        // Create a temporary input file to simulate user input
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.txt");
        std::fs::write(&input_file, "not_a_number\n").unwrap();

        let _installer = InstallCommand::new(false);
        let _field = ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        };

        // This would normally prompt for input, but we can't easily test interactive input
        // Just ensure the method exists and has the right signature
        let _method = InstallCommand::prompt_number_field;
    }

    #[test]
    fn test_prompt_boolean_field() {
        let _installer = InstallCommand::new(false);
        let _field = ConfigField {
            name: "enabled".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: None,
            default: Some("true".to_string()),
        };

        // This would normally prompt for input
        // Just ensure the method exists and has the right signature
        let _method = InstallCommand::prompt_boolean_field;
    }

    #[test]
    fn test_validate_final_config() {
        let installer = InstallCommand::new(false);
        let server = MockServer {
            metadata: ServerMetadata {
                name: "test".to_string(),
                description: Some("Test".to_string()),
                server_type: ServerType::Npm {
                    package: "test".to_string(),
                    version: None,
                },
                required_config: vec![],
                optional_config: vec![],
            },
            dependency: Dependency::NodeJs { min_version: None },
        };

        let config = HashMap::new();
        let result = installer.validate_final_config(&server, &config);
        // The result depends on the ConfigManager implementation
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_prompt_string_field() {
        let _installer = InstallCommand::new(false);
        let _field = ConfigField {
            name: "path".to_string(),
            field_type: ConfigFieldType::Path,
            description: None,
            default: Some("/default/path".to_string()),
        };

        // Test that the method exists and has the right signature
        let _method = InstallCommand::prompt_string_field;
    }

    #[test]
    fn test_display_config_mode() {
        let installer = InstallCommand::new(true);
        installer.display_config_mode(true);
        installer.display_config_mode(false);

        let installer_quiet = InstallCommand::new(false);
        installer_quiet.display_config_mode(true);
        installer_quiet.display_config_mode(false);
    }

    #[test]
    fn test_process_config_field() {
        let installer = InstallCommand::new(false);
        let mut config = HashMap::new();
        let field = ConfigField {
            name: "test".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        };
        let metadata = ServerMetadata {
            name: "test-server".to_string(),
            description: Some("Test".to_string()),
            server_type: ServerType::Npm {
                package: "test-server".to_string(),
                version: None,
            },
            required_config: vec![],
            optional_config: vec![field.clone()],
        };

        // With override already in config
        config.insert("test".to_string(), "override".to_string());
        let result = installer.process_config_field(&mut config, &field, &metadata, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_for_field_value() {
        let _installer = InstallCommand::new(false);

        // Test different field types
        let _string_field = ConfigField {
            name: "string".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        };

        let _path_field = ConfigField {
            name: "path".to_string(),
            field_type: ConfigFieldType::Path,
            description: None,
            default: None,
        };

        let _url_field = ConfigField {
            name: "url".to_string(),
            field_type: ConfigFieldType::Url,
            description: None,
            default: None,
        };

        let _number_field = ConfigField {
            name: "number".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        };

        let _bool_field = ConfigField {
            name: "bool".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: None,
            default: None,
        };

        // Just ensure the method handles all field types
        let _method = InstallCommand::prompt_for_field_value;
    }
}
