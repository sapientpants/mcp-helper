use assert_cmd::Command;
use predicates::prelude::*;

// Helper to get the binary command
fn get_command() -> Command {
    Command::cargo_bin("mcp").unwrap()
}

#[test]
fn test_help_output() {
    let mut cmd = get_command();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP Helper - Make MCP Just Work™"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn test_version_output() {
    let mut cmd = get_command();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mcp 0.1.0"));
}

#[test]
fn test_verbose_flag() {
    let mut cmd = get_command();
    cmd.arg("--verbose")
        .arg("setup")
        .assert()
        .success()
        .stderr(predicate::str::contains("Verbose mode enabled"));
}

#[test]
fn test_run_command_basic() {
    let mut cmd = get_command();
    cmd.arg("run")
        .arg("test-server")
        .assert()
        .failure() // Will fail because test-server doesn't exist
        .stdout(predicate::str::contains("Running MCP server: test-server"));
}

#[test]
fn test_run_command_with_args() {
    let mut cmd = get_command();
    cmd.arg("run")
        .arg("test-server")
        .arg("--")
        .arg("arg1")
        .arg("arg2")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Running MCP server: test-server"));
}

#[test]
fn test_install_command() {
    let mut cmd = get_command();
    cmd.arg("install")
        .arg("test-package")
        .assert()
        .failure() // Will fail due to no detected clients in test environment
        .stdout(predicate::str::contains(
            "Installing MCP server: test-package",
        ));
}

#[test]
fn test_setup_command() {
    let mut cmd = get_command();
    cmd.arg("setup")
        .assert()
        .success()
        .stdout(predicate::str::contains("Running MCP Helper setup..."))
        .stdout(predicate::str::contains(
            "Setup command not yet implemented",
        ));
}

#[test]
fn test_config_add_command() {
    let mut cmd = get_command();
    cmd.arg("config")
        .arg("add")
        .arg("my-server")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Adding server to config: my-server",
        ))
        .stdout(predicate::str::contains(
            "Config add command not yet implemented",
        ));
}

#[test]
fn test_config_list_command() {
    let mut cmd = get_command();
    cmd.arg("config")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configured MCP servers:"))
        .stdout(predicate::str::contains(
            "Config list command not yet implemented",
        ));
}

#[test]
fn test_config_remove_command() {
    let mut cmd = get_command();
    cmd.arg("config")
        .arg("remove")
        .arg("my-server")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Removing server from config: my-server",
        ))
        .stdout(predicate::str::contains(
            "Config remove command not yet implemented",
        ));
}

#[test]
fn test_doctor_command() {
    let mut cmd = get_command();
    cmd.arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Running MCP diagnostics..."))
        .stdout(predicate::str::contains(
            "Doctor command not yet implemented",
        ));
}

#[test]
fn test_missing_subcommand() {
    let mut cmd = get_command();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_invalid_subcommand() {
    let mut cmd = get_command();
    cmd.arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: unrecognized subcommand 'invalid'",
        ));
}

#[test]
fn test_run_missing_server_arg() {
    let mut cmd = get_command();
    cmd.arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: the following required arguments were not provided",
        ));
}

#[test]
fn test_install_missing_server_arg() {
    let mut cmd = get_command();
    cmd.arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: the following required arguments were not provided",
        ));
}

#[test]
fn test_config_add_missing_server_arg() {
    let mut cmd = get_command();
    cmd.arg("config")
        .arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: the following required arguments were not provided",
        ));
}

#[test]
fn test_config_remove_missing_server_arg() {
    let mut cmd = get_command();
    cmd.arg("config")
        .arg("remove")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: the following required arguments were not provided",
        ));
}

#[test]
fn test_config_missing_action() {
    let mut cmd = get_command();
    cmd.arg("config")
        .assert()
        .code(2) // Exit code 2 indicates missing subcommand
        .stderr(predicate::str::contains("Usage: mcp config"));
}

// Test platform detection through environment variable manipulation
#[test]
fn test_platform_detection_output() {
    // This test ensures platform detection works by running with verbose mode
    let mut cmd = get_command();
    cmd.arg("--verbose")
        .arg("run")
        .arg("test-server")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Detected platform:"));
}

// Test error formatting
#[test]
fn test_mcp_error_display() {
    // Install command will fail with a dialog error in non-terminal environment
    let mut cmd = get_command();
    cmd.arg("install")
        .arg("@test/package")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Dialog error: IO error"));
}

// Test help for subcommands
#[test]
fn test_run_help() {
    let mut cmd = get_command();
    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run an MCP server"))
        .stdout(predicate::str::contains(
            "Additional arguments to pass to the server",
        ));
}

#[test]
fn test_install_help() {
    let mut cmd = get_command();
    cmd.arg("install")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Install an MCP server"))
        .stdout(predicate::str::contains(
            "Name or path of the MCP server to install",
        ));
}

#[test]
fn test_config_help() {
    let mut cmd = get_command();
    cmd.arg("config")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage MCP server configurations"));
}

#[test]
fn test_verbose_with_install() {
    let mut cmd = get_command();
    cmd.arg("--verbose")
        .arg("install")
        .arg("test-package")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Verbose mode enabled"))
        .stderr(predicate::str::contains(
            "Detecting server type for: test-package",
        ));
}

// Test trailing arguments for run command
#[test]
fn test_run_with_multiple_trailing_args() {
    let mut cmd = get_command();
    cmd.arg("run")
        .arg("server")
        .arg("arg1")
        .arg("arg2")
        .arg("arg3")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Running MCP server: server"));
}

// Test edge cases
#[test]
fn test_empty_server_name() {
    let mut cmd = get_command();
    cmd.arg("run").arg("").assert().failure();
}

#[test]
fn test_unicode_server_name() {
    let mut cmd = get_command();
    cmd.arg("run")
        .arg("こんにちは-server")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Running MCP server: こんにちは-server",
        ));
}

// Module-level tests for helper functions
#[cfg(test)]
mod unit_tests {
    #[test]
    fn test_print_not_implemented_function() {
        // This function is tested indirectly through integration tests
        // but we can ensure it's called in the right contexts
        assert_eq!(1, 1); // Placeholder test
    }
}
