use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MCP Helper - Make MCP Just Work™"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("install"));
    assert!(stdout.contains("setup"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("doctor"));
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The binary name in clap is "mcp", not "mcp-helper"
    assert!(stdout.contains("mcp") || stdout.contains("mcp-helper"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_run_command_help() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "run", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Run an MCP server"));
    assert!(stdout.contains("Name of the MCP server to run"));
}

#[test]
fn test_config_subcommands() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "config", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("add"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("remove"));
}

#[test]
fn test_missing_subcommand() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--"])
        .output()
        .expect("Failed to execute command");

    // Should fail when no subcommand is provided
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("Usage"));
}

#[test]
fn test_run_without_server_name() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "run"])
        .output()
        .expect("Failed to execute command");

    // Should fail when server name is missing
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("required"));
}

#[test]
fn test_verbose_flag_position() {
    // Test verbose flag before subcommand
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "--verbose", "run", "test-server"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Verbose mode enabled"));

    // Test verbose flag after subcommand
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "run", "--verbose", "test-server"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Verbose mode enabled"));
}

#[test]
fn test_invalid_subcommand() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "invalid-command"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unrecognized subcommand") || stderr.contains("error"));
}

#[test]
fn test_config_add_without_server() {
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "config", "add"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_help_for_each_command() {
    let commands = ["run", "install", "setup", "config", "doctor"];

    for cmd in &commands {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", cmd, "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success(), "Help failed for command: {}", cmd);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "Help output is empty for command: {}",
            cmd
        );
    }
}
