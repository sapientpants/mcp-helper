//! Common utilities for E2E testing
//!
//! Provides helpers for binary execution, temporary directories,
//! and test environment setup.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

/// Test environment for E2E tests
pub struct TestEnvironment {
    /// Temporary directory for test files
    pub temp_dir: TempDir,
    /// Path to the MCP Helper binary
    pub binary_path: PathBuf,
    /// Environment variables for the test
    pub env_vars: HashMap<String, String>,
}

impl TestEnvironment {
    /// Create a new test environment
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        let binary_path = find_binary_path()?;

        let mut env_vars = HashMap::new();

        // Set up isolated environment
        env_vars.insert(
            "HOME".to_string(),
            temp_dir.path().to_string_lossy().to_string(),
        );
        env_vars.insert(
            "USERPROFILE".to_string(),
            temp_dir.path().to_string_lossy().to_string(),
        );

        // Disable actual config discovery during tests
        env_vars.insert("MCP_HELPER_TEST_MODE".to_string(), "1".to_string());

        Ok(Self {
            temp_dir,
            binary_path,
            env_vars,
        })
    }

    /// Get the path to the temporary directory
    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a file in the temporary directory
    pub fn create_file<P: AsRef<Path>>(&self, path: P, contents: &str) -> Result<PathBuf> {
        let full_path = self.temp_path().join(path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(&full_path, contents)
            .with_context(|| format!("Failed to write file: {}", full_path.display()))?;

        Ok(full_path)
    }

    /// Create a directory in the temporary directory
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let full_path = self.temp_path().join(path);
        fs::create_dir_all(&full_path)
            .with_context(|| format!("Failed to create directory: {}", full_path.display()))?;
        Ok(full_path)
    }

    /// Execute the MCP Helper binary with given arguments
    pub fn run_command(&self, args: &[&str]) -> Result<CommandResult> {
        let mut cmd = Command::new(&self.binary_path);
        cmd.args(args)
            .current_dir(self.temp_path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let output = cmd
            .output()
            .with_context(|| format!("Failed to execute command: {cmd:?}"))?;

        Ok(CommandResult {
            output,
            args: args.iter().map(|s| s.to_string()).collect(),
        })
    }

    /// Execute a command and expect it to succeed
    pub fn run_success(&self, args: &[&str]) -> Result<CommandResult> {
        let result = self.run_command(args)?;
        if !result.success() {
            anyhow::bail!(
                "Command failed: {:?}\nstdout: {}\nstderr: {}",
                result.args,
                result.stdout_string(),
                result.stderr_string()
            );
        }
        Ok(result)
    }

    /// Execute a command and expect it to fail
    pub fn run_failure(&self, args: &[&str]) -> Result<CommandResult> {
        let result = self.run_command(args)?;
        if result.success() {
            anyhow::bail!(
                "Command unexpectedly succeeded: {:?}\nstdout: {}",
                result.args,
                result.stdout_string()
            );
        }
        Ok(result)
    }

    /// Set an environment variable for subsequent commands
    #[allow(dead_code)]
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env_vars.insert(key.into(), value.into());
    }

    /// Create a mock MCP client config for testing
    pub fn create_mock_client_config(&self, client_name: &str) -> Result<PathBuf> {
        let config_dir = self.create_dir(format!("{client_name}_config"))?;

        let config_content = match client_name {
            "claude_desktop" => {
                r#"{
                "mcpServers": {}
            }"#
            }
            "vscode" => {
                r#"{
                "mcp": {
                    "servers": {}
                }
            }"#
            }
            _ => r#"{}"#,
        };

        let config_file = config_dir.join("config.json");
        fs::write(&config_file, config_content)
            .with_context(|| format!("Failed to write config file: {}", config_file.display()))?;

        Ok(config_file)
    }

    /// Create a mock NPM environment for testing
    #[allow(dead_code)]
    pub fn setup_mock_npm(&mut self) -> Result<()> {
        let npm_dir = self.create_dir("npm_global")?;
        let node_modules = npm_dir.join("node_modules");
        fs::create_dir_all(&node_modules)?;

        // Create a mock package.json
        let package_json = r#"{
            "name": "@modelcontextprotocol/server-filesystem",
            "version": "0.4.0",
            "description": "Mock MCP server for testing",
            "bin": {
                "mcp-server-filesystem": "./dist/index.js"
            }
        }"#;

        let package_dir = node_modules
            .join("@modelcontextprotocol")
            .join("server-filesystem");
        fs::create_dir_all(&package_dir)?;
        fs::write(package_dir.join("package.json"), package_json)?;

        // Create a mock executable
        let exec_content = if cfg!(windows) {
            "@echo off\necho Mock MCP server running"
        } else {
            "#!/bin/bash\necho \"Mock MCP server running\""
        };

        let exec_path = package_dir.join("dist").join("index.js");
        fs::create_dir_all(exec_path.parent().unwrap())?;
        fs::write(&exec_path, exec_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exec_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&exec_path, perms)?;
        }

        // Set NPM_CONFIG_PREFIX to use our mock npm directory
        self.env_vars.insert(
            "NPM_CONFIG_PREFIX".to_string(),
            npm_dir.to_string_lossy().to_string(),
        );

        Ok(())
    }
}

/// Result of executing a command
pub struct CommandResult {
    output: Output,
    args: Vec<String>,
}

impl CommandResult {
    /// Create a new CommandResult for testing
    #[cfg(test)]
    pub fn new_for_test(output: Output, args: Vec<String>) -> Self {
        Self { output, args }
    }

    /// Check if the command succeeded
    pub fn success(&self) -> bool {
        self.output.status.success()
    }

    /// Get the exit code
    pub fn exit_code(&self) -> Option<i32> {
        self.output.status.code()
    }

    /// Get stdout as a string
    pub fn stdout_string(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).to_string()
    }

    /// Get stderr as a string
    pub fn stderr_string(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).to_string()
    }

    /// Get stdout as bytes
    #[allow(dead_code)]
    pub fn stdout_bytes(&self) -> &[u8] {
        &self.output.stdout
    }

    /// Get stderr as bytes
    #[allow(dead_code)]
    pub fn stderr_bytes(&self) -> &[u8] {
        &self.output.stderr
    }

    /// Get the command arguments that were executed
    pub fn args(&self) -> &[String] {
        &self.args
    }
}

/// Find the path to the MCP Helper binary
fn find_binary_path() -> Result<PathBuf> {
    // First, try to find the binary in the target directory
    let cargo_manifest_dir =
        env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?;

    let project_root = PathBuf::from(cargo_manifest_dir);

    // Try debug build first, then release
    let debug_binary = project_root
        .join("target")
        .join("debug")
        .join(binary_name());
    let release_binary = project_root
        .join("target")
        .join("release")
        .join(binary_name());

    if debug_binary.exists() {
        return Ok(debug_binary);
    }

    if release_binary.exists() {
        return Ok(release_binary);
    }

    // Fall back to looking in PATH
    which::which("mcp").context("MCP Helper binary not found in target directory or PATH")
}

/// Get the binary name for the current platform
fn binary_name() -> &'static str {
    if cfg!(windows) {
        "mcp.exe"
    } else {
        "mcp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_creation() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        assert!(env.temp_path().exists());
        assert!(env.binary_path.exists() || env.binary_path.to_string_lossy().contains("mcp"));
    }

    #[test]
    fn test_file_creation() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        let file_path = env
            .create_file("test.txt", "Hello, world!")
            .expect("Failed to create file");

        assert!(file_path.exists());
        let contents = fs::read_to_string(&file_path).expect("Failed to read file");
        assert_eq!(contents, "Hello, world!");
    }

    #[test]
    fn test_directory_creation() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        let dir_path = env
            .create_dir("test_dir")
            .expect("Failed to create directory");

        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_mock_client_config() {
        let env = TestEnvironment::new().expect("Failed to create test environment");
        let config_path = env
            .create_mock_client_config("claude_desktop")
            .expect("Failed to create mock config");

        assert!(config_path.exists());
        let contents = fs::read_to_string(&config_path).expect("Failed to read config");
        assert!(contents.contains("mcpServers"));
    }
}
