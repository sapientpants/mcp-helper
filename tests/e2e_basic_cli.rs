//! Basic end-to-end tests for CLI functionality
//!
//! These tests verify basic CLI operations like help, version,
//! and error handling work correctly.

mod e2e;

use anyhow::Result;
use e2e::{
    assert_command_success, assert_help_text_formatted, assert_stderr_contains,
    assert_stdout_contains, assert_version_output, TestEnvironment,
};

#[test]
fn test_cli_help() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&["--help"])?;
    assert_help_text_formatted(&result);

    Ok(())
}

#[test]
fn test_cli_version() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&["--version"])?;
    assert_version_output(&result);

    Ok(())
}

#[test]
fn test_cli_no_args() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_command(&[])?;

    // The CLI shows help to stderr and exits with error code
    // This is expected behavior for missing required arguments
    let stderr = result.stderr_string();
    assert!(stderr.contains("Usage:") || stderr.contains("USAGE:"));
    assert!(stderr.contains("Commands:") || stderr.contains("COMMANDS:"));

    Ok(())
}

#[test]
fn test_cli_invalid_command() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_failure(&["invalid-command"])?;

    assert_stderr_contains(&result, "unrecognized");

    Ok(())
}

#[test]
fn test_cli_help_for_subcommands() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Test help for different subcommands
    let subcommands = ["add", "list", "remove", "setup", "doctor"];

    for subcommand in &subcommands {
        let result = env.run_success(&["help", subcommand])?;
        assert_help_text_formatted(&result);
        assert_stdout_contains(&result, subcommand);
    }

    Ok(())
}

#[test]
fn test_cli_verbose_flag() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&["--verbose", "--help"])?;

    // Should still show help, but potentially with more verbose output
    assert_help_text_formatted(&result);

    Ok(())
}

#[test]
fn test_cli_global_flags() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Test that global flags work with subcommands
    let result = env.run_success(&["--help", "add"])?;
    assert_help_text_formatted(&result);

    Ok(())
}

#[test]
fn test_cli_error_formatting() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Trigger an error to see if it's properly formatted
    let result = env.run_failure(&["add"])?; // Missing required argument

    let stderr = result.stderr_string();

    // Should have colored error output
    assert!(
        stderr.contains("error:") || stderr.contains("Error:") || stderr.contains("required"),
        "Error output doesn't contain expected error message: {stderr}"
    );

    Ok(())
}

#[test]
fn test_cli_startup_performance() -> Result<()> {
    let env = TestEnvironment::new()?;

    use std::time::Instant;

    let start = Instant::now();
    let _result = env.run_success(&["--version"])?;
    let duration = start.elapsed();

    // Should start up quickly (under 2 seconds for E2E test tolerance)
    assert!(
        duration.as_secs() < 2,
        "CLI startup took too long: {duration:?}"
    );

    Ok(())
}

#[test]
fn test_cli_output_encoding() -> Result<()> {
    let env = TestEnvironment::new()?;

    let result = env.run_success(&["--help"])?;

    let stdout = result.stdout_string();

    // Should handle Unicode characters properly
    assert!(stdout.is_ascii() || stdout.chars().all(|c| !c.is_control() || c.is_whitespace()));

    // Should have reasonable line lengths (not too wide)
    let max_line_length = stdout.lines().map(|line| line.len()).max().unwrap_or(0);
    assert!(
        max_line_length < 120,
        "Help text lines are too long: {max_line_length}"
    );

    Ok(())
}

#[test]
fn test_cli_exit_codes() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Success case
    let success_result = env.run_success(&["--version"])?;
    assert_eq!(success_result.exit_code(), Some(0));

    // Failure case
    let failure_result = env.run_failure(&["invalid-command"])?;
    assert!(failure_result.exit_code().unwrap_or(0) != 0);

    Ok(())
}

#[test]
fn test_cli_signal_handling() -> Result<()> {
    // This is a basic test - more complex signal testing would require
    // process spawning and signal sending
    let env = TestEnvironment::new()?;

    // Just verify the CLI can start and stop normally
    let result = env.run_success(&["--version"])?;
    assert_command_success(&result);

    Ok(())
}

#[test]
fn test_cli_environment_isolation() -> Result<()> {
    let mut env = TestEnvironment::new()?;

    // Set some environment variables
    let temp_path_str = env.temp_path().to_string_lossy().to_string();
    env.set_env("MCP_TEST_VAR", "test_value");
    env.set_env("HOME", temp_path_str);

    let result = env.run_success(&["--version"])?;
    assert_command_success(&result);

    // CLI should work regardless of environment
    Ok(())
}

#[test]
fn test_cli_concurrent_execution() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Run multiple commands concurrently to test for race conditions
    use std::thread;

    let handles: Vec<_> = (0..3)
        .map(|_| {
            let env_path = env.binary_path.clone();
            thread::spawn(move || {
                let mut cmd = std::process::Command::new(&env_path);
                cmd.arg("--version");
                cmd.output()
            })
        })
        .collect();

    for handle in handles {
        let output = handle.join().unwrap()?;
        assert!(output.status.success());
    }

    Ok(())
}

#[test]
fn test_cli_working_directory() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a subdirectory and run from there
    let subdir = env.create_dir("subdir")?;

    let mut cmd = std::process::Command::new(&env.binary_path);
    cmd.arg("--version").current_dir(&subdir);

    let output = cmd.output()?;
    assert!(output.status.success());

    Ok(())
}
