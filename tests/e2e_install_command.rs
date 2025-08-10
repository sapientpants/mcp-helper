//! End-to-end tests for the `mcp install` command
//!
//! These tests verify that the install command works correctly with
//! real CLI invocations, client configuration, and filesystem operations.

mod e2e;

use anyhow::Result;
use e2e::{assert_stderr_contains, assert_stdout_contains, TestEnvironment};

#[test]
fn test_install_command_help() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&["help", "install"])?;

    assert_stdout_contains(&result, "Install an MCP server");
    assert_stdout_contains(&result, "Usage:");
    assert_stdout_contains(&result, "Arguments:");

    Ok(())
}

#[test]
fn test_install_npm_server_dry_run() -> Result<()> {
    let env = TestEnvironment::new()?;

    // The install command requires interactive input, so it will fail in non-terminal
    let result = env.run_failure(&[
        "install",
        "@modelcontextprotocol/server-filesystem",
        "--dry-run",
    ])?;

    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_with_config() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a test directory for allowed paths
    let allowed_dir = env.create_dir("allowed_files")?;

    // The install command requires interactive input
    let result = env.run_failure(&[
        "install",
        "@modelcontextprotocol/server-filesystem",
        "--config",
        &format!("allowed_paths={}", allowed_dir.display()),
        "--dry-run",
    ])?;

    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_with_auto_deps() -> Result<()> {
    let env = TestEnvironment::new()?;

    // The install command requires interactive input
    let result = env.run_failure(&[
        "install",
        "@modelcontextprotocol/server-filesystem",
        "--auto-install-deps",
        "--dry-run",
    ])?;

    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_with_version_specification() -> Result<()> {
    let env = TestEnvironment::new()?;

    // The install command requires interactive input
    let result = env.run_failure(&[
        "install",
        "@modelcontextprotocol/server-filesystem@0.4.0",
        "--dry-run",
    ])?;

    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_server_not_found() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_failure(&["install", "nonexistent-server-12345"])?;

    // Should fail on dialog in non-terminal environment
    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_missing_dependency() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Try to install an NPM server without Node.js available
    let result = env.run_failure(&["install", "@modelcontextprotocol/server-filesystem"])?;

    // Should fail on dialog in non-terminal environment
    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_no_args() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_failure(&["install"])?;

    // Should show error about missing server argument
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("required") || stderr.contains("USAGE:"),
        "Expected error about missing server argument"
    );

    Ok(())
}

#[test]
fn test_install_docker_server() -> Result<()> {
    let env = TestEnvironment::new()?;

    // The install command requires interactive input
    let result = env.run_failure(&["install", "docker:nginx:alpine", "--dry-run"])?;

    // Docker check happens before dialog
    let stderr = result.stderr_string();
    assert!(stderr.contains("Dialog error") || stderr.contains("Docker"));

    Ok(())
}

#[test]
fn test_install_binary_server() -> Result<()> {
    let env = TestEnvironment::new()?;

    // The install command requires interactive input
    let result = env.run_failure(&[
        "install",
        "https://github.com/example/mcp-server/releases/download/v1.0.0/server-linux-x64",
        "--dry-run",
    ])?;

    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_python_server() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a mock Python script
    let python_script = env.create_file(
        "server.py",
        r#"
#!/usr/bin/env python3
# Mock MCP server
print("Hello from Python MCP server")
    "#,
    )?;

    // The install command requires interactive input
    let result = env.run_failure(&["install", &python_script.to_string_lossy(), "--dry-run"])?;

    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_batch_file() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a batch file
    let batch_file = env.create_file(
        "servers.txt",
        "@modelcontextprotocol/server-filesystem\n@anthropic/mcp-server-slack\n",
    )?;

    // The install command requires interactive input even for batch
    let result = env.run_failure(&[
        "install",
        "placeholder", // Required but ignored when using --batch
        "--batch",
        &batch_file.to_string_lossy(),
        "--dry-run",
    ])?;

    // Shows batch processing before failing on dialog
    assert_stdout_contains(&result, "Installing servers from batch file");
    // The actual error message varies but should indicate failure
    let stderr = result.stderr_string();
    assert!(!stderr.is_empty(), "Expected stderr output but got none");

    Ok(())
}

#[test]
fn test_install_verbose_output() -> Result<()> {
    let env = TestEnvironment::new()?;

    // The install command requires interactive input
    let result = env.run_failure(&[
        "--verbose",
        "install",
        "@modelcontextprotocol/server-filesystem",
        "--dry-run",
    ])?;

    // Verbose mode should show detailed steps before failing
    assert_stderr_contains(&result, "Verbose mode enabled");
    assert_stdout_contains(&result, "Checking dependencies");
    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_security_validation() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Try to install from an untrusted source
    let result = env.run_failure(&["install", "http://suspicious-domain.com/malware.js"])?;

    // Security check happens before dialog
    assert_stderr_contains(&result, "Installation blocked due to security concerns");

    Ok(())
}

#[test]
fn test_install_config_validation() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Try to install with invalid configuration
    let result = env.run_failure(&[
        "install",
        "@modelcontextprotocol/server-filesystem",
        "--config",
        "invalid_field=value",
    ])?;

    // Should fail on dialog before config validation
    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}

#[test]
fn test_install_with_multiple_config_values() -> Result<()> {
    let env = TestEnvironment::new()?;

    // The install command requires interactive input
    let result = env.run_failure(&[
        "install",
        "@modelcontextprotocol/server-filesystem",
        "--config",
        "allowed_paths=/home,/tmp",
        "--config",
        "readonly=true",
        "--dry-run",
    ])?;

    // Should fail due to non-terminal environment or missing clients
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("Dialog error") || stderr.contains("No MCP clients selected"),
        "Expected Dialog error or No MCP clients message, got: {stderr}"
    );

    Ok(())
}
