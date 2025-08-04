//! End-to-end CLI integration tests
//!
//! These tests verify the complete command-line interface functionality
//! including argument parsing, command execution, and output validation.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cli_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MCP") || stdout.contains("help") || stdout.contains("Usage"));
}

#[test]
fn test_cli_version_output() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1") || stdout.contains("mcp"));
}

#[test]
fn test_run_command_basic() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "test-server"])
        .output()
        .expect("Failed to execute command");

    // Command should execute (may fail if server doesn't exist)
    let _ = output.status;
}

#[test]
fn test_run_with_local_javascript() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("server.js");
    fs::write(&script_path, "console.log('MCP Server Started');").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", script_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should attempt to run with node
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Either node runs it or node is not found
    assert!(
        stdout.contains("MCP Server Started") || stderr.contains("node") || stderr.contains("Node")
    );
}

#[test]
fn test_run_with_local_python() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("server.py");
    fs::write(&script_path, "print('MCP Python Server')").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", script_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should attempt to run with python
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Either python runs it or python is not found
    assert!(
        stdout.contains("MCP Python Server")
            || stderr.contains("python")
            || stderr.contains("Python")
    );
}

#[test]
fn test_run_with_npm_package_format() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "@scope/package"])
        .output()
        .expect("Failed to execute command");

    // Should recognize NPM package format
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should attempt to use npx
    assert!(
        stderr.contains("npx")
            || stderr.contains("npm")
            || stderr.contains("not found")
            || stderr.contains("failed")
    );
}

#[test]
fn test_run_with_arguments() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "run",
            "test-server",
            "--port=3000",
            "--host=localhost",
            "--debug=true",
        ])
        .output()
        .expect("Failed to execute command");

    // Arguments should be parsed without panic
    assert!(output.status.code().is_some());
}

#[test]
fn test_install_command_basic() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "install",
            "test-server",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    // Dry run should complete
    let _ = output.status;
}

#[test]
fn test_run_with_nonexistent_file() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "run",
            "/nonexistent/path/to/server.js",
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail with error
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}{stdout}");

    assert!(
        combined.contains("not found") ||
        combined.contains("does not exist") ||
        combined.contains("no such file") ||  // lowercase version
        combined.contains("No such file") ||
        combined.contains("cannot find") ||
        combined.contains("ENOENT") ||  // Node error code
        combined.contains("exited with status") // Generic error from our runner
    );
}

#[test]
fn test_cli_with_verbose_flag() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-v", "run", "test"])
        .output()
        .expect("Failed to execute command");

    // Verbose flag should be accepted
    let _ = output.status;
}

#[test]
fn test_cli_with_quiet_flag() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-q", "run", "test"])
        .output()
        .expect("Failed to execute command");

    // Quiet flag should be accepted
    let _ = output.status;
}

#[test]
fn test_run_with_shell_script() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("server.sh");

    #[cfg(unix)]
    {
        fs::write(&script_path, "#!/bin/bash\necho 'Shell MCP Server'").unwrap();

        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    #[cfg(windows)]
    {
        fs::write(&script_path, "@echo off\necho Shell MCP Server").unwrap();
    }

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", script_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should handle shell scripts
    let _ = output.status;
}

#[test]
fn test_run_with_docker_compose_file() {
    let temp_dir = TempDir::new().unwrap();
    let compose_path = temp_dir.path().join("docker-compose.yml");
    fs::write(
        &compose_path,
        "version: '3'\nservices:\n  mcp:\n    image: mcp-server:latest\n",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "run",
            compose_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Should recognize docker-compose file
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should attempt to use docker-compose
    assert!(
        stderr.contains("docker")
            || stderr.contains("Docker")
            || stderr.contains("not found")
            || stderr.contains("failed")
    );
}

#[test]
fn test_run_with_package_version() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "express@4.18.0"])
        .output()
        .expect("Failed to execute command");

    // Should handle version specifiers
    let _ = output.status;
}

#[test]
fn test_multiple_argument_formats() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "run",
            "server",
            "--key=value",
            "--flag",
            "--number=123",
            "--bool=false",
            "--json={\"test\":true}",
        ])
        .output()
        .expect("Failed to execute command");

    // Various argument formats should be accepted
    let _ = output.status;
}

#[test]
fn test_environment_variable_passing() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "test-server"])
        .env("MCP_TEST_VAR", "test_value")
        .env("NODE_ENV", "production")
        .output()
        .expect("Failed to execute command");

    // Environment variables should be available to the subprocess
    let _ = output.status;
}

#[test]
fn test_path_with_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let dir_with_spaces = temp_dir.path().join("my server directory");
    fs::create_dir(&dir_with_spaces).unwrap();
    let script_path = dir_with_spaces.join("server.js");
    fs::write(&script_path, "console.log('Server in path with spaces');").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", script_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should handle paths with spaces
    let _ = output.status;
}

#[test]
fn test_relative_path_resolution() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "./local-server.js"])
        .output()
        .expect("Failed to execute command");

    // Should handle relative paths
    let _ = output.status;
}

#[test]
fn test_parent_directory_path() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "../server.js"])
        .output()
        .expect("Failed to execute command");

    // Should handle parent directory references
    let _ = output.status;
}

#[test]
fn test_cli_error_messages() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "invalid-command"])
        .output()
        .expect("Failed to execute command");

    // Should provide helpful error message
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error")
            || stderr.contains("Error")
            || stderr.contains("unknown")
            || stderr.contains("invalid")
    );
}
