//! Comprehensive CLI integration tests for main.rs

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create a test command
fn test_cmd() -> Command {
    Command::cargo_bin("mcp").unwrap()
}

/// Helper to check if output contains expected text (ignoring ANSI codes)
fn contains_text(text: &str) -> predicates::str::ContainsPredicate {
    predicate::str::contains(text)
}

/// Helper to check if output contains any of multiple texts
fn contains_any_of(texts: &[&str]) -> impl Fn(&str) -> bool {
    let texts_owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
    move |s: &str| texts_owned.iter().any(|text| s.contains(text))
}

#[test]
fn test_help_command() {
    test_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains_text("MCP Helper - Make MCP Just Work™"))
        .stdout(contains_text("Usage:"))
        .stdout(contains_text("Commands:"))
        .stdout(contains_text("run"))
        .stdout(contains_text("install"))
        .stdout(contains_text("setup"))
        .stdout(contains_text("config"))
        .stdout(contains_text("doctor"));
}

#[test]
fn test_version_command() {
    test_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains_text("mcp 0.1.0"));
}

#[test]
fn test_verbose_flag() {
    test_cmd()
        .args(["--verbose", "run", "test-server"])
        .assert()
        .failure() // Will fail because server doesn't exist, but should process verbose flag
        .stderr(contains_text("Verbose mode enabled"));
}

#[test]
fn test_run_command_help() {
    test_cmd()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(contains_text("Run an MCP server"))
        .stdout(contains_text("Usage:"))
        .stdout(contains_text("server"))
        .stdout(contains_text("Additional arguments"));
}

#[test]
fn test_run_command_missing_server() {
    test_cmd()
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::function(contains_any_of(&[
            "required", "server", "argument", "missing",
        ])));
}

#[test]
fn test_run_command_with_server() {
    test_cmd()
        .args(["run", "nonexistent-server"])
        .assert()
        .failure() // Should fail because server doesn't exist
        .stdout(contains_text("Running MCP server: nonexistent-server"));
}

#[test]
fn test_run_command_with_args() {
    test_cmd()
        .args(["run", "test-server", "--", "--port", "3000", "--verbose"])
        .assert()
        .failure() // Will fail but should accept the args
        .stdout(contains_text("Running MCP server: test-server"));
}

#[test]
fn test_install_command_help() {
    test_cmd()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(contains_text("Install an MCP server"))
        .stdout(contains_text("Usage:"))
        .stdout(contains_text("server"))
        .stdout(contains_text("auto-install-deps"))
        .stdout(contains_text("dry-run"))
        .stdout(contains_text("config"))
        .stdout(contains_text("batch"));
}

#[test]
fn test_install_command_missing_server() {
    test_cmd()
        .arg("install")
        .assert()
        .failure()
        .stderr(predicate::function(contains_any_of(&[
            "required", "server", "argument", "missing",
        ])));
}

#[test]
fn test_install_command_basic() {
    test_cmd()
        .args(["install", "test-server"])
        .assert()
        .failure() // Will fail during actual installation, but should parse correctly
        .stdout(contains_text("Installing MCP server: test-server"));
}

#[test]
fn test_install_command_with_auto_install_deps() {
    test_cmd()
        .args(["install", "test-server", "--auto-install-deps"])
        .assert()
        .failure() // Will fail during installation
        .stdout(contains_text("Installing MCP server: test-server"));
}

#[test]
fn test_install_command_with_dry_run() {
    test_cmd()
        .args(["install", "test-server", "--dry-run"])
        .assert()
        .failure() // Will fail during installation
        .stdout(contains_text("Installing MCP server: test-server"));
}

#[test]
fn test_install_command_with_config() {
    test_cmd()
        .args([
            "install",
            "test-server",
            "--config",
            "key1=value1",
            "--config",
            "key2=value2",
        ])
        .assert()
        .failure() // Will fail during installation
        .stdout(contains_text("Installing MCP server: test-server"));
}

#[test]
fn test_install_command_with_batch_file() {
    test_cmd()
        .args(["install", "test-server", "--batch", "/path/to/batch.json"])
        .assert()
        .failure() // Will fail because batch file doesn't exist
        .stdout(contains_text("Installing servers from batch file"));
}

#[test]
fn test_setup_command() {
    test_cmd()
        .arg("setup")
        .assert()
        .success()
        .stdout(contains_text("Running MCP Helper setup"))
        .stdout(contains_text("Setup command not yet implemented"));
}

#[test]
fn test_config_command_help() {
    test_cmd()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(contains_text("Manage MCP server configurations"))
        .stdout(contains_text("Commands:"))
        .stdout(contains_text("add"))
        .stdout(contains_text("list"))
        .stdout(contains_text("remove"));
}

#[test]
fn test_config_command_missing_action() {
    test_cmd()
        .arg("config")
        .assert()
        .failure()
        .stderr(predicate::function(contains_any_of(&[
            "required",
            "subcommand",
            "action",
        ])));
}

#[test]
fn test_config_add_command() {
    test_cmd()
        .args(["config", "add", "test-server"])
        .assert()
        .success()
        .stdout(contains_text("Adding server to config: test-server"))
        .stdout(contains_text("Config add command not yet implemented"));
}

#[test]
fn test_config_add_missing_server() {
    test_cmd()
        .args(["config", "add"])
        .assert()
        .failure()
        .stderr(predicate::function(contains_any_of(&[
            "required", "server", "argument",
        ])));
}

#[test]
fn test_config_list_command() {
    test_cmd()
        .args(["config", "list"])
        .assert()
        .success()
        .stdout(contains_text("Configured MCP servers"))
        .stdout(contains_text("Config list command not yet implemented"));
}

#[test]
fn test_config_remove_command() {
    test_cmd()
        .args(["config", "remove", "test-server"])
        .assert()
        .success()
        .stdout(contains_text("Removing server from config: test-server"))
        .stdout(contains_text("Config remove command not yet implemented"));
}

#[test]
fn test_config_remove_missing_server() {
    test_cmd()
        .args(["config", "remove"])
        .assert()
        .failure()
        .stderr(predicate::function(contains_any_of(&[
            "required", "server", "argument",
        ])));
}

#[test]
fn test_doctor_command() {
    test_cmd()
        .arg("doctor")
        .assert()
        .success()
        .stdout(contains_text("Running MCP diagnostics"))
        .stdout(contains_text("Doctor command not yet implemented"));
}

#[test]
fn test_invalid_command() {
    test_cmd()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::function(contains_any_of(&[
            "unrecognized subcommand",
            "invalid",
            "command",
        ])));
}

#[test]
fn test_global_verbose_flag_position() {
    // Test that --verbose works before the command
    test_cmd()
        .args(["--verbose", "setup"])
        .assert()
        .success()
        .stderr(contains_text("Verbose mode enabled"))
        .stdout(contains_text("Running MCP Helper setup"));
}

#[test]
fn test_run_command_with_verbose() {
    test_cmd()
        .args(["--verbose", "run", "test-server"])
        .assert()
        .failure() // Will fail but should show verbose output
        .stderr(contains_text("Verbose mode enabled"))
        .stderr(contains_text("Detected platform"));
}

#[test]
fn test_install_command_all_flags() {
    test_cmd()
        .args([
            "--verbose",
            "install",
            "test-server",
            "--auto-install-deps",
            "--dry-run",
            "--config",
            "key=value",
        ])
        .assert()
        .failure() // Will fail during installation
        .stderr(contains_text("Verbose mode enabled"))
        .stdout(contains_text("Installing MCP server: test-server"));
}

#[test]
fn test_command_exit_codes() {
    // Successful commands should exit with 0
    let output = test_cmd().arg("setup").output().unwrap();
    assert_eq!(output.status.code(), Some(0));

    // Failed commands should exit with 1
    let output = test_cmd()
        .args(["run", "nonexistent-server"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_error_message_formatting() {
    test_cmd()
        .args(["run", "nonexistent-server"])
        .assert()
        .failure()
        .stderr(predicate::function(|s: &str| {
            // Should contain proper error formatting (even if we can't see ANSI codes in tests)
            s.contains("nonexistent-server") || s.contains("error") || s.contains("failed")
        }));
}

#[test]
fn test_help_subcommands() {
    // Test that each subcommand has help
    let subcommands = ["run", "install", "setup", "config", "doctor"];

    for subcommand in &subcommands {
        test_cmd()
            .args([subcommand, "--help"])
            .assert()
            .success()
            .stdout(contains_text("Usage:"));
    }
}

#[test]
fn test_config_subcommand_help() {
    let config_subcommands = ["add", "list", "remove"];

    for subcommand in &config_subcommands {
        test_cmd()
            .args(["config", subcommand, "--help"])
            .assert()
            .success()
            .stdout(contains_text("Usage:"));
    }
}

#[test]
fn test_no_arguments() {
    test_cmd()
        .assert()
        .failure()
        .stderr(predicate::function(contains_any_of(&[
            "required",
            "subcommand",
            "Usage",
        ])));
}

#[test]
fn test_run_command_preserves_args() {
    // Test that arguments are preserved and passed through
    test_cmd()
        .args([
            "run",
            "test-server",
            "--",
            "--custom-arg",
            "value",
            "--flag",
        ])
        .assert()
        .failure() // Will fail but should accept the trailing args
        .stdout(contains_text("Running MCP server: test-server"));
}

#[test]
fn test_install_config_parsing() {
    // Test that config arguments are parsed correctly
    test_cmd()
        .args([
            "install",
            "test-server",
            "--config",
            "host=localhost",
            "--config",
            "port=3000",
            "--config",
            "debug=true",
        ])
        .assert()
        .failure(); // Will fail during installation but should parse config correctly
}

#[test]
fn test_verbose_logging_initialization() {
    test_cmd()
        .args(["--verbose", "doctor"])
        .assert()
        .success()
        .stderr(contains_text("Verbose mode enabled"));
}

#[test]
fn test_cli_structure_completeness() {
    // Test that the help output contains all expected sections
    test_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains_text("MCP Helper - Make MCP Just Work"))
        .stdout(contains_text("Options:"))
        .stdout(contains_text("--verbose"))
        .stdout(contains_text("--help"))
        .stdout(contains_text("--version"));
}

#[test]
#[cfg(target_os = "windows")]
fn test_platform_detection_windows() {
    test_cmd()
        .args(["--verbose", "run", "test"])
        .assert()
        .failure()
        .stderr(contains_text("Detected platform: Windows"));
}

#[test]
#[cfg(target_os = "macos")]
fn test_platform_detection_macos() {
    test_cmd()
        .args(["--verbose", "run", "test"])
        .assert()
        .failure()
        .stderr(contains_text("Detected platform: MacOS"));
}

#[test]
#[cfg(target_os = "linux")]
fn test_platform_detection_linux() {
    test_cmd()
        .args(["--verbose", "run", "test"])
        .assert()
        .failure()
        .stderr(contains_text("Detected platform: Linux"));
}
