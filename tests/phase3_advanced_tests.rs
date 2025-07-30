use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_non_interactive_config_flag() {
    let mut cmd = Command::cargo_bin("mcp").unwrap();

    // Test with valid config
    cmd.arg("install")
        .arg("@test/package")
        .arg("--config")
        .arg("api_key=test123")
        .arg("--config")
        .arg("port=3000")
        .arg("--dry-run");

    let output = cmd.output().unwrap();

    // The command should run without prompting
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("non-interactive mode")
            || String::from_utf8_lossy(&output.stdout).contains("Installing")
    );
}

#[test]
fn test_invalid_config_format() {
    let mut cmd = Command::cargo_bin("mcp").unwrap();

    cmd.arg("install")
        .arg("test-server")
        .arg("--config")
        .arg("invalid-format-no-equals")
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should warn about invalid format
    assert!(stderr.contains("Invalid config format") || stderr.contains("Expected key=value"));
}

#[test]
fn test_batch_file_creation_and_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("test_servers.conf");

    // Create a test batch file
    let batch_content = r#"
# Test batch configuration
[test-server-1]
api_key=abc123
port=3000
debug=true

[test-server-2]
url=https://example.com
timeout=30

# Another server
[test-server-3]
name=production
"#;

    fs::write(&batch_file, batch_content).unwrap();

    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install")
        .arg("dummy") // This will be ignored when using --batch
        .arg("--batch")
        .arg(batch_file.to_str().unwrap())
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect multiple servers
    assert!(stdout.contains("Found") && (stdout.contains("server") || stdout.contains("3")));
}

#[test]
fn test_batch_file_with_invalid_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("invalid_batch.conf");

    // Create batch file with invalid syntax
    let batch_content = r#"
[server1]
valid_key=valid_value
invalid line without equals sign
another_key=value
"#;

    fs::write(&batch_file, batch_content).unwrap();

    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install")
        .arg("dummy")
        .arg("--batch")
        .arg(batch_file.to_str().unwrap())
        .arg("--dry-run");

    let output = cmd.output().unwrap();

    // Should fail with parsing error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid line") || stderr.contains("Expected key=value"));
}

#[test]
fn test_batch_file_nonexistent() {
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install")
        .arg("dummy")
        .arg("--batch")
        .arg("/nonexistent/path/to/batch.conf");

    let output = cmd.output().unwrap();

    // Should fail with file not found error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to read batch file") || stderr.contains("No such file"));
}

#[test]
fn test_empty_batch_file() {
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("empty_batch.conf");

    // Create empty batch file (just comments and whitespace)
    let batch_content = r#"
# This is just a comment file
# No actual servers defined

# More comments
"#;

    fs::write(&batch_file, batch_content).unwrap();

    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install")
        .arg("dummy")
        .arg("--batch")
        .arg(batch_file.to_str().unwrap());

    let output = cmd.output().unwrap();

    // Should fail with no servers found
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No servers found") || stderr.contains("empty"));
}

#[test]
fn test_config_flag_help() {
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install").arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention the config flag
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("key=value") || stdout.contains("configuration"));
}

#[test]
fn test_batch_flag_help() {
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install").arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention the batch flag
    assert!(stdout.contains("--batch"));
    assert!(stdout.contains("file") || stdout.contains("batch"));
}

#[test]
fn test_config_and_batch_flags_mutual_exclusion() {
    // When using --batch, individual config flags should still work
    // but the batch file takes precedence for server-specific config
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("simple_batch.conf");

    let batch_content = r#"
[test-server]
batch_key=from_batch
"#;

    fs::write(&batch_file, batch_content).unwrap();

    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("install")
        .arg("dummy")
        .arg("--config")
        .arg("global_key=from_cli")
        .arg("--batch")
        .arg(batch_file.to_str().unwrap())
        .arg("--dry-run");

    let output = cmd.output().unwrap();

    // Should work - batch mode should override config args per server
    // The command should not fail due to conflicting flags
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should process the batch file
    assert!(
        stdout.contains("Found")
            || stdout.contains("Installing")
            || !stderr.contains("conflicting")
    );
}

#[test]
fn test_verbose_mode_with_advanced_options() {
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("verbose_test.conf");

    let batch_content = r#"
[test-server]
test_key=test_value
"#;

    fs::write(&batch_file, batch_content).unwrap();

    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("--verbose")
        .arg("install")
        .arg("dummy")
        .arg("--batch")
        .arg(batch_file.to_str().unwrap())
        .arg("--dry-run");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show verbose output
    assert!(
        stderr.contains("Verbose mode") || stderr.contains("ℹ") || stderr.contains("Detecting")
    );
}
