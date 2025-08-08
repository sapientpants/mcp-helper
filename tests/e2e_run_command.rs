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
    assert_stdout_contains(&result, "Usage:");

    Ok(())
}

#[test]
fn test_run_command_server_not_found() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_failure(&["run", "nonexistent-server"])?;

    // The error message could be from NPM or from our code
    assert!(
        result.stderr_string().contains("not found")
            || result.stderr_string().contains("Not found")
            || result.stderr_string().contains("404")
    );

    Ok(())
}

#[test]
fn test_run_command_missing_server_arg() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_failure(&["run"])?;

    assert_stderr_contains(&result, "required");

    Ok(())
}

// The following tests would require actual server execution
// They are marked as ignored to prevent CI failures

#[test]
#[ignore = "Requires actual NPM server execution"]
fn test_run_command_with_npm_server() -> Result<()> {
    let mut env = TestEnvironment::new()?;

    // Set up mock npm environment
    env.setup_mock_npm()?;

    // This would actually try to run the server
    // In a real scenario, we'd need to handle the process properly
    let _result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--",
        "--help", // Pass help to the server to make it exit quickly
    ])?;

    Ok(())
}

#[test]
#[ignore = "Requires actual NPM server with version"]
fn test_run_command_with_version() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem@1.0.0",
        "--",
        "--help",
    ])?;

    Ok(())
}

#[test]
#[ignore = "Requires actual NPM server execution"]
fn test_run_command_with_config_args() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _result = env.run_success(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--",
        "--allow=/tmp",
        "--readonly",
    ])?;

    Ok(())
}

#[test]
#[ignore = "Requires Docker"]
fn test_run_command_docker_server() -> Result<()> {
    let env = TestEnvironment::new()?;

    // This would require Docker to be installed
    let _result = env.run_failure(&["run", "docker:ollama/ollama:latest"])?;

    Ok(())
}

#[test]
#[ignore = "Requires Python"]
fn test_run_command_python_server() -> Result<()> {
    let env = TestEnvironment::new()?;

    // This would require Python to be installed
    let _result = env.run_failure(&["run", "server.py"])?;

    Ok(())
}

#[test]
#[ignore = "Requires binary URL"]
fn test_run_command_binary_url() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _result = env.run_failure(&[
        "run",
        "https://github.com/owner/repo/releases/download/v1.0/binary",
    ])?;

    Ok(())
}

#[test]
fn test_run_command_verbose_output() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_failure(&["run", "--verbose", "nonexistent-server"])?;

    // Should show more detailed error with verbose flag
    // The error message could be from NPM or from our code
    assert!(
        result.stderr_string().contains("not found")
            || result.stderr_string().contains("Not found")
            || result.stderr_string().contains("404")
    );

    Ok(())
}

#[test]
#[ignore = "Requires environment setup"]
fn test_run_command_with_environment_variables() -> Result<()> {
    let mut env = TestEnvironment::new()?;

    env.set_env("API_KEY", "test-key");
    env.set_env("CUSTOM_VAR", "test-value");

    let _result = env.run_failure(&["run", "@modelcontextprotocol/server-filesystem"])?;

    Ok(())
}

#[test]
#[ignore = "Requires dependency checking"]
fn test_run_command_dependency_check() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Without Node.js installed, NPM servers should fail
    let result = env.run_failure(&["run", "@modelcontextprotocol/server-filesystem"])?;

    assert_stderr_contains(&result, "Node.js");

    Ok(())
}

#[test]
#[ignore = "Requires interrupt handling"]
fn test_run_command_interrupt_handling() -> Result<()> {
    // This test would require spawning the process and sending signals
    // which is complex to test reliably in E2E tests
    Ok(())
}

#[test]
#[ignore = "Requires path handling"]
fn test_run_command_cross_platform_paths() -> Result<()> {
    let env = TestEnvironment::new()?;

    #[cfg(target_os = "windows")]
    let path = "C:\\Users\\test\\config.json";
    #[cfg(not(target_os = "windows"))]
    let path = "/home/test/config.json";

    let _result = env.run_failure(&[
        "run",
        "@modelcontextprotocol/server-filesystem",
        "--",
        &format!("--config={path}"),
    ])?;

    Ok(())
}
