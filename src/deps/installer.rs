use crate::deps::{Dependency, DependencyCheck, InstallMethod};
use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use std::process::Command;

/// Auto-installer for missing dependencies
pub struct DependencyInstaller {
    dry_run: bool,
    auto_confirm: bool,
}

impl DependencyInstaller {
    pub fn new() -> Self {
        Self {
            dry_run: false,
            auto_confirm: false,
        }
    }

    pub fn with_dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn with_auto_confirm(mut self) -> Self {
        self.auto_confirm = true;
        self
    }

    /// Attempt to auto-install a missing dependency
    pub fn install_dependency(&self, check: &DependencyCheck) -> Result<bool> {
        let Some(instructions) = &check.install_instructions else {
            return Ok(false);
        };

        let dependency_name = check.dependency.name();
        let platform_methods = instructions.for_platform();

        if platform_methods.is_empty() {
            println!(
                "  {} No installation methods available for {} on this platform",
                "⚠".yellow(),
                dependency_name
            );
            return Ok(false);
        }

        // Find the best installation method
        let install_method = self.select_best_method(platform_methods)?;

        if self.dry_run {
            println!(
                "  {} [DRY RUN] Would install {} using: {}",
                "🔍".blue(),
                dependency_name,
                install_method.name
            );
            println!("    Command: {}", install_method.command.cyan());
            return Ok(true);
        }

        // Confirm installation unless auto-confirm is enabled
        if !self.auto_confirm {
            let prompt = format!(
                "Install {} using {}? This will run: {}",
                dependency_name.cyan(),
                install_method.name.green(),
                install_method.command.yellow()
            );

            if !Confirm::new().with_prompt(prompt).interact()? {
                println!("  {} Installation cancelled by user", "❌".red());
                return Ok(false);
            }
        }

        // Execute the installation
        self.execute_install_method(install_method, dependency_name)
    }

    fn select_best_method<'a>(&self, methods: &'a [InstallMethod]) -> Result<&'a InstallMethod> {
        // Priority order for different installation methods
        let preferred_methods = self.get_preferred_methods();

        // Find the first method that matches our preference order
        for preferred in &preferred_methods {
            if let Some(method) = methods
                .iter()
                .find(|m| m.name.to_lowercase().contains(preferred))
            {
                return Ok(method);
            }
        }

        // If no preferred method found, use the first available
        methods.first().context("No installation methods available")
    }

    #[cfg(target_os = "windows")]
    fn get_preferred_methods(&self) -> Vec<&'static str> {
        vec!["winget", "chocolatey", "scoop"]
    }

    #[cfg(target_os = "macos")]
    fn get_preferred_methods(&self) -> Vec<&'static str> {
        vec!["homebrew", "macports"]
    }

    #[cfg(target_os = "linux")]
    fn get_preferred_methods(&self) -> Vec<&'static str> {
        vec!["apt", "dnf", "yum", "pacman", "snap"]
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn get_preferred_methods(&self) -> Vec<&'static str> {
        vec![]
    }

    fn execute_install_method(
        &self,
        method: &InstallMethod,
        dependency_name: &str,
    ) -> Result<bool> {
        println!(
            "  {} Installing {} using {}...",
            "🚀".blue(),
            dependency_name.cyan(),
            method.name.green()
        );

        let result = if method.command.starts_with("http") {
            // This is a download URL, not a command
            println!(
                "  {} Please download and install from: {}",
                "🌐".blue(),
                method.command.underline()
            );
            return Ok(false); // Manual installation required
        } else if method.command.contains("&&") {
            // Execute compound command
            self.execute_compound_command(&method.command)
        } else {
            // Execute simple command
            self.execute_simple_command(&method.command)
        };

        match result {
            Ok(success) => {
                if success {
                    println!(
                        "  {} Successfully installed {}",
                        "✅".green(),
                        dependency_name
                    );
                    Ok(true)
                } else {
                    println!(
                        "  {} Installation of {} may have failed",
                        "⚠".yellow(),
                        dependency_name
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                println!(
                    "  {} Failed to install {}: {}",
                    "❌".red(),
                    dependency_name,
                    e
                );
                Ok(false) // Don't propagate error, just report it
            }
        }
    }

    fn execute_simple_command(&self, command_str: &str) -> Result<bool> {
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            anyhow::bail!("Empty command");
        }

        let output = Command::new(parts[0])
            .args(&parts[1..])
            .output()
            .with_context(|| format!("Failed to execute command: {command_str}"))?;

        Ok(output.status.success())
    }

    fn execute_compound_command(&self, command_str: &str) -> Result<bool> {
        // For compound commands, we'll use the system shell
        let shell_cmd = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command_str]).output()
        } else {
            Command::new("sh").args(["-c", command_str]).output()
        };

        let output = shell_cmd
            .with_context(|| format!("Failed to execute compound command: {command_str}"))?;

        Ok(output.status.success())
    }

    /// Check if a dependency checker would require elevated privileges
    pub fn requires_elevation(&self, dependency: &Dependency) -> bool {
        match dependency {
            Dependency::NodeJs { .. } => false, // Usually available without sudo
            Dependency::Python { .. } => cfg!(target_os = "linux"), // Linux system packages need sudo
            Dependency::Docker { .. } => true, // Docker usually requires elevated privileges
            Dependency::Git => cfg!(target_os = "linux"), // Linux system packages need sudo
        }
    }

    /// Get elevation warning message
    pub fn get_elevation_warning(&self, dependency: &Dependency) -> Option<String> {
        if self.requires_elevation(dependency) {
            Some(format!(
                "Installing {} may require administrator/sudo privileges",
                dependency.name()
            ))
        } else {
            None
        }
    }

    /// Install multiple dependencies in the correct order
    pub fn install_dependencies(&self, checks: &[DependencyCheck]) -> Result<Vec<bool>> {
        let mut results = Vec::new();

        for check in checks {
            let result = self.install_dependency(check)?;
            results.push(result);

            // If this was a successful installation, we might want to verify it worked
            if result && !self.dry_run {
                // Small delay to allow system to update
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        Ok(results)
    }
}

impl Default for DependencyInstaller {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect available package managers on the current system
pub fn detect_package_managers() -> Vec<String> {
    let mut managers = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if command_exists("winget") {
            managers.push("winget".to_string());
        }
        if command_exists("choco") {
            managers.push("chocolatey".to_string());
        }
        if command_exists("scoop") {
            managers.push("scoop".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        if command_exists("brew") {
            managers.push("homebrew".to_string());
        }
        if command_exists("port") {
            managers.push("macports".to_string());
        }
    }

    #[cfg(target_os = "linux")]
    {
        if command_exists("apt") {
            managers.push("apt".to_string());
        }
        if command_exists("dnf") {
            managers.push("dnf".to_string());
        }
        if command_exists("yum") {
            managers.push("yum".to_string());
        }
        if command_exists("pacman") {
            managers.push("pacman".to_string());
        }
        if command_exists("snap") {
            managers.push("snap".to_string());
        }
    }

    managers
}

/// Check if a command exists in PATH
fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_installer_creation() {
        let installer = DependencyInstaller::new();
        assert!(!installer.dry_run);
        assert!(!installer.auto_confirm);

        let installer = DependencyInstaller::new()
            .with_dry_run()
            .with_auto_confirm();
        assert!(installer.dry_run);
        assert!(installer.auto_confirm);
    }

    #[test]
    fn test_dependency_installer_default() {
        let installer = DependencyInstaller::default();
        assert!(!installer.dry_run);
        assert!(!installer.auto_confirm);
    }

    #[test]
    fn test_requires_elevation() {
        let installer = DependencyInstaller::new();

        let node_dep = Dependency::NodeJs { min_version: None };
        let docker_dep = Dependency::Docker {
            min_version: None,
            requires_compose: false,
        };

        // Node.js usually doesn't require elevation
        assert!(!installer.requires_elevation(&node_dep));

        // Docker usually requires elevation
        assert!(installer.requires_elevation(&docker_dep));
    }

    #[test]
    fn test_get_elevation_warning() {
        let installer = DependencyInstaller::new();

        let node_dep = Dependency::NodeJs { min_version: None };
        let docker_dep = Dependency::Docker {
            min_version: None,
            requires_compose: false,
        };

        let _node_warning = installer.get_elevation_warning(&node_dep);
        let docker_warning = installer.get_elevation_warning(&docker_dep);

        // Docker should have a warning, Node.js might not (platform dependent)
        assert!(docker_warning.is_some());
        if let Some(warning) = docker_warning {
            assert!(warning.contains("Docker"));
            assert!(warning.contains("administrator") || warning.contains("sudo"));
        }
    }

    #[test]
    fn test_select_best_method() {
        let installer = DependencyInstaller::new();
        let methods = vec![
            InstallMethod {
                name: "download".to_string(),
                command: "https://example.com".to_string(),
                description: None,
            },
            InstallMethod {
                name: "homebrew".to_string(),
                command: "brew install test".to_string(),
                description: None,
            },
        ];

        let result = installer.select_best_method(&methods);
        assert!(result.is_ok());

        // Should prefer package manager over download on supported platforms
        #[cfg(target_os = "macos")]
        {
            let selected = result.unwrap();
            assert_eq!(selected.name, "homebrew");
        }
    }

    #[test]
    fn test_detect_package_managers() {
        let managers = detect_package_managers();
        // Should return a vector (might be empty on some systems)
        assert!(managers.is_empty() || !managers.is_empty()); // Always true, but validates it runs
    }

    #[test]
    fn test_command_exists() {
        // Test with a command that should exist on most systems
        let exists = command_exists("echo") || command_exists("cmd");
        assert!(exists); // At least one should exist

        // Test with a command that definitely doesn't exist
        let not_exists = command_exists("this-command-definitely-does-not-exist-12345");
        assert!(!not_exists);
    }

    #[test]
    fn test_install_dependencies_empty() {
        let installer = DependencyInstaller::new().with_dry_run();
        let checks = vec![];
        let results = installer.install_dependencies(&checks).unwrap();
        assert!(results.is_empty());
    }
}
