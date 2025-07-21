use std::process::Command;

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
