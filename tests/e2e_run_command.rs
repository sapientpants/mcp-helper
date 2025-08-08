//! End-to-end tests for the `mcp run` command
//!
//! These tests verify that the run command works correctly with
//! real CLI invocations and filesystem operations.

mod e2e;

use anyhow::Result;
use e2e::{assert_stderr_contains, assert_stdout_contains, TestEnvironment};

#[test]
fn test_run_command_help() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&["help", "run"])?;

    assert_stdout_contains(&result, "Run an MCP server");
    assert_stdout_contains(&result, "USAGE:");

    Ok(())
}

#[test]
fn test_run_command_with_npm_server() -> Result<()> {
    let mut env = TestEnvironment::new()?;

    // Set up mock npm environment
    env.setup_mock_npm()?;

    let result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--dry-run", // Don't actually start the server
    ])?;

    assert_stdout_contains(&result, "Would execute:");
    assert_stdout_contains(&result, "npx");
    assert_stdout_contains(&result, "@modelcontextprotocol/server-filesystem");

    Ok(())
}

#[test]
fn test_run_command_server_not_found() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_failure(&["run", "nonexistent-server"])?;

    assert_stderr_contains(&result, "Server not found");
    assert_stderr_contains(&result, "nonexistent-server");

    Ok(())
}

#[test]
fn test_run_command_no_args() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_failure(&["run"])?;

    // Should show error about missing server argument
    let stderr = result.stderr_string();
    assert!(
        stderr.contains("required") || stderr.contains("USAGE:"),
        "Expected error about missing server argument"
    );

    Ok(())
}

#[test]
fn test_run_command_with_version() -> Result<()> {
    let mut env = TestEnvironment::new()?;
    env.setup_mock_npm()?;

    let result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem@1.0.0",
        "--dry-run",
    ])?;

    assert_stdout_contains(&result, "Would execute:");
    assert_stdout_contains(&result, "@modelcontextprotocol/server-filesystem");

    Ok(())
}

#[test]
fn test_run_command_with_config_args() -> Result<()> {
    let mut env = TestEnvironment::new()?;
    env.setup_mock_npm()?;

    let result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--dry-run",
        "--",
        "--allow=/tmp",
        "--readonly",
    ])?;

    assert_stdout_contains(&result, "Would execute:");
    assert_stdout_contains(&result, "--allow=/tmp");
    assert_stdout_contains(&result, "--readonly");

    Ok(())
}

#[test]
fn test_run_command_verbose_output() -> Result<()> {
    let mut env = TestEnvironment::new()?;
    env.setup_mock_npm()?;

    let result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--verbose",
        "--dry-run",
    ])?;

    // Verbose output should include more details
    assert_stdout_contains(&result, "Would execute:");

    Ok(())
}

#[test]
fn test_run_command_docker_server() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&["run", "docker:ollama/ollama:latest", "--dry-run"])?;

    assert_stdout_contains(&result, "Would execute:");
    assert_stdout_contains(&result, "docker");
    assert_stdout_contains(&result, "ollama/ollama:latest");

    Ok(())
}

#[test]
fn test_run_command_binary_url() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&[
        "run",
        "https://github.com/owner/repo/releases/download/v1.0/binary",
        "--dry-run",
    ])?;

    assert_stdout_contains(&result, "Would execute:");
    assert_stdout_contains(&result, "https://github.com");

    Ok(())
}

#[test]
fn test_run_command_python_server() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&["run", "server.py", "--dry-run"])?;

    assert_stdout_contains(&result, "Would execute:");
    assert_stdout_contains(&result, "python");
    assert_stdout_contains(&result, "server.py");

    Ok(())
}

#[test]
fn test_run_command_with_environment_variables() -> Result<()> {
    let mut env = TestEnvironment::new()?;
    env.setup_mock_npm()?;

    // Set environment variables
    env.set_env("API_KEY", "test-key");
    env.set_env("DEBUG", "true");

    let result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--dry-run",
    ])?;

    assert_stdout_contains(&result, "Would execute:");

    Ok(())
}

#[test]
fn test_run_command_cross_platform_paths() -> Result<()> {
    let mut env = TestEnvironment::new()?;
    env.setup_mock_npm()?;

    // Test with a path that needs normalization
    let result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--dry-run",
        "--",
        #[cfg(windows)]
        "--config=C:\\Users\\test\\config.json",
        #[cfg(not(windows))]
        "--config=/home/test/config.json",
    ])?;

    assert_stdout_contains(&result, "Would execute:");
    assert_stdout_contains(&result, "config");

    Ok(())
}

#[test]
fn test_run_command_interrupt_handling() -> Result<()> {
    // This test just verifies the command starts correctly
    // Actual interrupt handling is hard to test in E2E
    let env = TestEnvironment::new()?;

    let result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--dry-run",
    ])?;

    assert_stdout_contains(&result, "Would execute:");

    Ok(())
}

#[test]
fn test_run_command_dependency_check() -> Result<()> {
    // For now, we just verify the command structure is correct
    let mut env = TestEnvironment::new()?;
    env.setup_mock_npm()?;

    let result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--dry-run",
    ])?;

    assert_stdout_contains(&result, "Would execute:");

    Ok(())
}
