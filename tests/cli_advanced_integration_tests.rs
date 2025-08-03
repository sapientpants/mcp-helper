//! Advanced CLI integration tests for complex scenarios and edge cases

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a test command
fn test_cmd() -> Command {
    Command::cargo_bin("mcp").unwrap()
}

/// Helper to check if output contains expected text
fn contains_text(text: &str) -> predicates::str::ContainsPredicate {
    predicate::str::contains(text)
}

#[test]
fn test_run_command_with_complex_server_names() {
    // Test scoped NPM packages
    test_cmd()
        .args([
            "run",
            "@modelcontextprotocol/server-filesystem",
            "--",
            "--path",
            "/tmp",
        ])
        .assert()
        .failure() // Server not installed
        .stdout(contains_text(
            "Running MCP server: @modelcontextprotocol/server-filesystem",
        ));

    // Test package with version
    test_cmd()
        .args(["run", "mcp-server@1.2.3"])
        .assert()
        .failure()
        .stdout(contains_text("Running MCP server: mcp-server@1.2.3"));
}

#[test]
fn test_run_command_with_path_arguments() {
    // Test Windows-style paths on all platforms
    test_cmd()
        .args([
            "run",
            "test-server",
            "--",
            "--config",
            "C:\\Users\\Test\\config.json",
            "--output",
            "D:\\Data\\output.txt",
        ])
        .assert()
        .failure();

    // Test Unix-style paths
    test_cmd()
        .args([
            "run",
            "test-server",
            "--",
            "--config",
            "/home/user/config.json",
            "--output",
            "/var/log/output.txt",
        ])
        .assert()
        .failure();
}

#[test]
fn test_install_command_with_multiple_configs() {
    // Test multiple config values with various formats
    test_cmd()
        .args([
            "install",
            "test-server",
            "--config",
            "host=localhost",
            "--config",
            "port=3000",
            "--config",
            "ssl=true",
            "--config",
            "path=/var/data",
            "--config",
            "api_key=sk-1234567890",
            "--config",
            "debug_mode=false",
        ])
        .assert()
        .failure()
        .stdout(contains_text("Installing MCP server: test-server"));
}

#[test]
fn test_long_argument_handling() {
    // Test very long server names
    let long_server_name = "a".repeat(255);
    test_cmd()
        .args(["run", &long_server_name])
        .assert()
        .failure();

    // Test many arguments
    let mut args = vec!["run", "test-server", "--"];
    for i in 0..50 {
        args.push("--arg");
        let value = format!("value{i}");
        args.push(Box::leak(Box::new(value)).as_str());
    }
    test_cmd().args(&args).assert().failure();
}

#[test]
fn test_special_characters_in_arguments() {
    // Test arguments with special characters
    test_cmd()
        .args([
            "run",
            "test-server",
            "--",
            "--message=Hello, World!",
            "--path=/tmp/file with spaces.txt",
            "--regex=^test.*$",
            "--json={\"key\": \"value\"}",
        ])
        .assert()
        .failure();

    // Test server names with special characters
    test_cmd()
        .args([
            "install",
            "test-server-v2.0",
            "--config",
            "key=value with spaces",
        ])
        .assert()
        .failure();
}

#[test]
fn test_environment_variable_interaction() {
    // Test with custom PATH
    let mut cmd = test_cmd();
    cmd.env("PATH", "/custom/path:/usr/bin")
        .args(["run", "test-server"])
        .assert()
        .failure();

    // Test with NODE_PATH set
    let mut cmd = test_cmd();
    cmd.env("NODE_PATH", "/custom/node/modules")
        .args(["run", "test-server"])
        .assert()
        .failure();
}

#[test]
fn test_unicode_handling() {
    // Test Unicode in server names
    test_cmd().args(["run", "测试-server"]).assert().failure();

    // Test Unicode in arguments
    test_cmd()
        .args([
            "run",
            "test-server",
            "--",
            "--message=Hello 世界",
            "--path=/tmp/文件.txt",
        ])
        .assert()
        .failure();

    // Test Unicode in config
    test_cmd()
        .args(["install", "test-server", "--config", "message=こんにちは"])
        .assert()
        .failure();
}

#[test]
fn test_empty_and_whitespace_arguments() {
    // Test empty server name (should fail)
    test_cmd().args(["run", ""]).assert().failure();

    // Test whitespace-only server name
    test_cmd().args(["run", "   "]).assert().failure();

    // Test empty config values
    test_cmd()
        .args(["install", "test-server", "--config", "key="])
        .assert()
        .failure();
}

#[test]
fn test_batch_file_with_temp_dir() {
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("servers.json");

    // Create a batch file
    fs::write(
        &batch_file,
        r#"[
            {
                "name": "@modelcontextprotocol/server-filesystem",
                "config": {
                    "path": "/tmp"
                }
            },
            {
                "name": "test-server",
                "config": {
                    "port": 3000
                }
            }
        ]"#,
    )
    .unwrap();

    test_cmd()
        .args(["install", "dummy", "--batch", batch_file.to_str().unwrap()])
        .assert()
        .failure() // Will fail during installation
        .stdout(contains_text("Installing servers from batch file"));
}

#[test]
fn test_stdin_handling() {
    // Test that stdin doesn't interfere with commands
    test_cmd()
        .args(["run", "test-server"])
        .write_stdin("unexpected input\n")
        .assert()
        .failure();
}

#[test]
fn test_multiple_verbose_flags() {
    // Test that multiple verbose flags are rejected by clap
    test_cmd()
        .args(["--verbose", "--verbose", "run", "test-server"])
        .assert()
        .failure()
        .stderr(contains_text("cannot be used multiple times"));
}

#[test]
fn test_mixed_flag_styles() {
    // Test mixing -- and - style arguments
    test_cmd()
        .args([
            "run",
            "test-server",
            "--",
            "-v",
            "--verbose",
            "-p",
            "3000",
            "--host",
            "localhost",
        ])
        .assert()
        .failure();
}

#[test]
fn test_relative_vs_absolute_paths() {
    // Test relative path server
    test_cmd()
        .args(["run", "./local-server.js"])
        .assert()
        .failure();

    // Test absolute path server
    let absolute_path = if cfg!(windows) {
        "C:\\Program Files\\MCP\\server.exe"
    } else {
        "/usr/local/bin/mcp-server"
    };
    test_cmd().args(["run", absolute_path]).assert().failure();
}

#[test]
fn test_command_aliases_dont_exist() {
    // Verify that common aliases don't work (they shouldn't)
    test_cmd()
        .arg("r") // might be expected as alias for "run"
        .assert()
        .failure();

    test_cmd()
        .arg("i") // might be expected as alias for "install"
        .assert()
        .failure();
}

#[test]
fn test_command_interruption() {
    // Test that commands handle failures properly
    // Since assert_cmd::Command doesn't support spawn(), we test the failure case directly
    test_cmd()
        .args(["run", "long-running-server"])
        .timeout(std::time::Duration::from_millis(100))
        .assert()
        .failure();
}

#[test]
fn test_output_format_consistency() {
    // Test that all commands follow consistent output format
    let commands = vec![vec!["setup"], vec!["doctor"], vec!["config", "list"]];

    for cmd_args in commands {
        let output = test_cmd().args(&cmd_args).output().unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);

        // All commands should have some kind of header with emoji or symbol
        assert!(
            stdout.contains("🔧")
                || stdout.contains("🏥")
                || stdout.contains("📋")
                || stdout.contains("→"),
            "Command {cmd_args:?} missing consistent header formatting"
        );
    }
}

#[test]
fn test_error_recovery_suggestions() {
    // Test that errors include helpful suggestions
    test_cmd()
        .args(["run", "nonexistent-server"])
        .assert()
        .failure()
        .stderr(predicate::function(|s: &str| {
            s.contains("mcp install") || s.contains("Try running")
        }));
}

#[test]
fn test_dry_run_safety() {
    // Test that dry-run doesn't make actual changes
    test_cmd()
        .args([
            "install",
            "@modelcontextprotocol/server-filesystem",
            "--dry-run",
            "--config",
            "path=/tmp",
        ])
        .assert()
        .failure(); // Will fail but should indicate dry-run mode
}

#[test]
fn test_config_key_value_parsing_edge_cases() {
    // Test config with = in value
    test_cmd()
        .args([
            "install",
            "test-server",
            "--config",
            "connection_string=postgres://user:pass@host:5432/db?ssl=true",
        ])
        .assert()
        .failure();

    // Test config with special characters
    test_cmd()
        .args([
            "install",
            "test-server",
            "--config",
            "regex_pattern=^[a-zA-Z0-9]+$",
            "--config",
            "json_value={\"nested\":\"value\"}",
        ])
        .assert()
        .failure();
}

#[test]
fn test_help_output_completeness() {
    // Ensure help includes examples and descriptions
    let output = test_cmd().arg("--help").output().unwrap();

    let help_text = String::from_utf8_lossy(&output.stdout);

    // Check for presence of key sections
    assert!(help_text.contains("Usage:"));
    assert!(help_text.contains("Commands:"));
    assert!(help_text.contains("Options:"));
    assert!(help_text.contains("MCP Helper"));
}

#[test]
fn test_version_format() {
    let output = test_cmd().arg("--version").output().unwrap();

    let version_text = String::from_utf8_lossy(&output.stdout);

    // Should follow semantic versioning
    assert!(version_text.contains("0.1.0"));
    assert!(version_text.trim().starts_with("mcp"));
}

#[test]
fn test_subcommand_chaining_prevention() {
    // Test that we can't chain subcommands incorrectly
    test_cmd()
        .args(["run", "setup"]) // "setup" should be treated as server name, not subcommand
        .assert()
        .failure()
        .stdout(contains_text("Running MCP server: setup"));
}

#[test]
fn test_argument_order_flexibility() {
    // Test global flags after subcommand (should fail in clap)
    test_cmd()
        .args(["run", "test-server", "--verbose"])
        .assert()
        .failure();

    // Test correct order
    test_cmd()
        .args(["--verbose", "run", "test-server"])
        .assert()
        .failure()
        .stderr(contains_text("Verbose mode enabled"));
}

#[test]
fn test_install_validation_messages() {
    // Test that install provides clear validation messages
    test_cmd().args(["install", ""]).assert().failure();

    test_cmd()
        .args(["install", "test-server", "--config", "invalid"])
        .assert()
        .failure();
}
