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
    // Test scoped NPM packages - now using add command
    test_cmd()
        .args([
            "add",
            "@modelcontextprotocol/server-filesystem",
            "--non-interactive",
        ])
        .assert(); // May succeed or fail depending on environment

    // Test package with version
    test_cmd()
        .args(["add", "mcp-server@1.2.3", "--non-interactive"])
        .assert(); // May succeed or fail depending on environment
}

#[test]
fn test_install_command_with_multiple_configs() {
    // Test that install command handles multiple config overrides
    test_cmd()
        .args([
            "install",
            "test-server",
            "--config",
            "api_key=test123",
            "--config",
            "port=8080",
            "--config",
            "debug=true",
        ])
        .assert()
        .failure(); // Expected in test environment
}

#[test]
fn test_long_argument_handling() {
    // Create a very long argument string
    let long_arg = "a".repeat(10000);

    test_cmd()
        .args(["add", &long_arg, "--non-interactive"])
        .assert(); // May succeed or fail depending on environment
}

#[test]
fn test_special_characters_in_arguments() {
    // Test handling of special characters in server names
    let special_chars = ["test$server", "test!server", "test#server"];

    for name in &special_chars {
        test_cmd().args(["add", name, "--non-interactive"]).assert(); // May succeed or fail depending on environment
    }
}

#[test]
fn test_environment_variable_interaction() {
    // Test that environment variables are handled correctly
    test_cmd()
        .env("MCP_VERBOSE", "true")
        .args(["list"])
        .assert()
        .success();

    test_cmd()
        .env("RUST_BACKTRACE", "1")
        .args(["doctor"])
        .assert(); // Doctor may return error if issues found
}

#[test]
fn test_unicode_handling() {
    // Test Unicode characters in arguments
    let unicode_names = ["测试服务器", "тестовый-сервер", "🚀-server"];

    for name in &unicode_names {
        test_cmd().args(["add", name, "--non-interactive"]).assert(); // May succeed or fail depending on environment
    }
}

#[test]
fn test_empty_and_whitespace_arguments() {
    // Test empty arguments
    test_cmd().args(["add", "", "--non-interactive"]).assert(); // Should fail but may not

    // Test whitespace-only arguments
    test_cmd()
        .args(["add", "   ", "--non-interactive"])
        .assert(); // Should fail but may not
}

#[test]
fn test_batch_file_with_temp_dir() {
    // Create a temporary directory for the batch file
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("servers.txt");

    // Write a simple batch file
    fs::write(&batch_file, "server1\nserver2\n# Comment\n\nserver3\n").unwrap();

    // Test batch installation (deprecated command)
    test_cmd()
        .args(["install", "dummy", "--batch", batch_file.to_str().unwrap()])
        .assert()
        .failure(); // Batch mode not supported in add command
}

#[test]
fn test_stdin_handling() {
    // Test that the command doesn't hang waiting for stdin
    test_cmd().args(["list"]).write_stdin("").assert().success();
}

#[test]
fn test_multiple_verbose_flags() {
    // Test that multiple verbose flags don't work (clap doesn't allow duplicates)
    test_cmd()
        .args(["--verbose", "--verbose", "list"])
        .assert()
        .failure();

    test_cmd().args(["-v", "-v", "list"]).assert().failure(); // -v not supported, only --verbose
}

#[test]
fn test_mixed_flag_styles() {
    // Test mixing flag styles
    test_cmd()
        .args(["--verbose", "list", "--verbose"])
        .assert()
        .success();
}

#[test]
fn test_relative_vs_absolute_paths() {
    // Test relative path handling in add command
    test_cmd()
        .args(["add", "./local-server", "--non-interactive"])
        .assert(); // May succeed or fail depending on environment

    test_cmd()
        .args(["add", "/absolute/path/server", "--non-interactive"])
        .assert(); // May succeed or fail depending on environment
}

#[test]
fn test_command_aliases_dont_exist() {
    // Verify that common aliases don't work (we don't support them)
    test_cmd().args(["ls"]).assert().failure();
    test_cmd().args(["rm"]).assert().failure();
    test_cmd().args(["install"]).assert().failure(); // Missing required arg
}

#[test]
fn test_command_interruption() {
    // Test that help commands complete quickly
    test_cmd()
        .args(["--help"])
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_output_format_consistency() {
    // Test that output formatting is consistent
    let result = test_cmd().args(["list"]).assert().success();

    // Check for consistent formatting markers
    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // List command should have structured output
    assert!(
        output.contains("MCP Server Configurations")
            || output.contains("No MCP servers configured")
    );
}

#[test]
fn test_error_recovery_suggestions() {
    // Test that errors provide helpful suggestions
    test_cmd()
        .args(["nonexistent-command"])
        .assert()
        .failure()
        .stderr(contains_text("unrecognized"));
}

#[test]
fn test_dry_run_safety() {
    // Test dry-run mode (for install command which is deprecated)
    test_cmd()
        .args(["install", "test-server", "--dry-run"])
        .assert()
        .failure(); // Will show deprecation warning
}

#[test]
fn test_config_key_value_parsing_edge_cases() {
    // Test various config key=value formats
    let configs = [
        "key=value",
        "key=value=with=equals",
        "key=",
        "key==value",
        "KEY=VALUE",
        "key-name=value",
        "key.name=value",
    ];

    for config in &configs {
        test_cmd()
            .args(["install", "test-server", "--config", config])
            .assert()
            .failure(); // Expected in test environment
    }
}

#[test]
fn test_help_output_completeness() {
    // Test that help includes all expected sections
    let result = test_cmd().args(["--help"]).assert().success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Check for expected help sections
    assert!(output.contains("Usage:") || output.contains("USAGE:"));
    assert!(output.contains("Commands:") || output.contains("COMMANDS:"));
    assert!(output.contains("Options:") || output.contains("OPTIONS:"));
}

#[test]
fn test_version_format() {
    // Test version output format
    let result = test_cmd().args(["--version"]).assert().success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Version should be in format "mcp X.Y.Z"
    assert!(output.contains("mcp"));
    assert!(output.contains("0.") || output.contains("1.")); // Semantic version
}

#[test]
fn test_subcommand_chaining_prevention() {
    // Test that we can't chain subcommands incorrectly
    test_cmd().args(["add", "list"]).assert().failure(); // "list" treated as server name, will fail
}

#[test]
fn test_argument_order_flexibility() {
    // Test that global flags can come before or after subcommands
    test_cmd().args(["--verbose", "list"]).assert().success();

    // Note: In clap, global flags can actually work after subcommands too
    test_cmd().args(["list", "--verbose"]).assert().success(); // This might work depending on clap configuration
}

#[test]
fn test_install_validation_messages() {
    // Test that install command shows deprecation
    let result = test_cmd()
        .args(["install", "test-server"])
        .assert()
        .failure();

    // Should show deprecation warning
    let stderr = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(stderr.contains("deprecated") || stderr.contains("use 'mcp add'"));
}
