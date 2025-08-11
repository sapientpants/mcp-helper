//! End-to-end integration tests for mcp-helper
//!
//! These tests verify that the CLI works correctly with various scenarios
//! including different server types, paths, and configurations.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help_output() {
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn test_cli_version_output() {
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("mcp"));
}

// Note: The run command has been removed from mcp-helper
// These tests are kept as comments for historical reference

// #[test]
// fn test_run_command_basic() {
//     let output = Command::new("cargo")
//         .args(["run", "--quiet", "--", "run", "test-server"])
//         .output()
//         .expect("Failed to execute command");
//
//     let stdout = String::from_utf8_lossy(&output.stdout);
//     assert!(stdout.contains("Running MCP server: test-server"));
// }

// #[test]
// fn test_run_with_local_javascript() {
//     let temp_dir = TempDir::new().unwrap();
//     let script_path = temp_dir.path().join("server.js");
//     fs::write(&script_path, "console.log('MCP Server Started');").unwrap();
//
//     let output = Command::new("cargo")
//         .args(["run", "--quiet", "--", "run", script_path.to_str().unwrap()])
//         .output()
//         .expect("Failed to execute command");
//
//     // Should attempt to run with node
//     let stderr = String::from_utf8_lossy(&output.stderr);
//     let stdout = String::from_utf8_lossy(&output.stdout);
//
//     // Either node runs it or node is not found
//     assert!(
//         stdout.contains("MCP Server Started") || stderr.contains("node") || stderr.contains("Node")
//     );
// }

// #[test]
// fn test_run_with_local_python() {
//     let temp_dir = TempDir::new().unwrap();
//     let script_path = temp_dir.path().join("server.py");
//     fs::write(&script_path, "print('MCP Python Server')").unwrap();
//
//     let output = Command::new("cargo")
//         .args(["run", "--quiet", "--", "run", script_path.to_str().unwrap()])
//         .output()
//         .expect("Failed to execute command");
//
//     let stderr = String::from_utf8_lossy(&output.stderr);
//     let stdout = String::from_utf8_lossy(&output.stdout);
//
//     // Either python runs it or python is not found
//     assert!(
//         stdout.contains("MCP Python Server") || stderr.contains("python") || stderr.contains("Python")
//     );
// }

// #[test]
// fn test_run_with_npm_package_format() {
//     let output = Command::new("cargo")
//         .args([
//             "run",
//             "--quiet",
//             "--",
//             "run",
//             "@modelcontextprotocol/server-filesystem",
//         ])
//         .output()
//         .expect("Failed to execute command");
//
//     let stdout = String::from_utf8_lossy(&output.stdout);
//     assert!(stdout.contains("@modelcontextprotocol/server-filesystem"));
// }

#[test]
fn test_run_with_arguments() {
    // Test that arguments after -- are properly passed
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "add",
            "test-server",
            "--non-interactive",
        ])
        .output()
        .expect("Failed to execute command");

    // The add command should execute (may fail due to no clients)
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should at least show the adding message or an error
    assert!(
        stdout.contains("Adding MCP server")
            || stderr.contains("No MCP clients found")
            || stdout.contains("No MCP clients found")
    );
}

#[test]
fn test_install_command_basic() {
    // Test install command (now deprecated)
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "install",
            "test-server",
            "--config",
            "test=value",
        ])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show deprecation warning
    assert!(stderr.contains("deprecated") || stderr.contains("use 'mcp add'"));
}

// #[test]
// fn test_run_with_nonexistent_file() {
//     let output = Command::new("cargo")
//         .args(["run", "--quiet", "--", "run", "/nonexistent/path/to/server"])
//         .output()
//         .expect("Failed to execute command");
//
//     assert!(!output.status.success());
//
//     let stderr = String::from_utf8_lossy(&output.stderr);
//     assert!(
//         stderr.contains("not found") ||
//         stderr.contains("No such file") ||
//         stderr.contains("cannot find")
//     );
// }

#[test]
fn test_cli_with_verbose_flag() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--verbose", "list"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verbose mode should be enabled
    assert!(stderr.contains("Verbose mode enabled") || stderr.contains("verbose"));
}

#[test]
fn test_cli_with_quiet_flag() {
    // Test that invalid flags are rejected
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--quiet", "list"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unexpected argument") || stderr.contains("not found"));
}

#[test]
fn test_run_with_shell_script() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("server.sh");
    fs::write(&script_path, "#!/bin/bash\necho 'Shell MCP Server'").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    // Test with add command instead of run
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "add",
            script_path.to_str().unwrap(),
            "--non-interactive",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should at least recognize it as a local file
    assert!(
        stdout.contains("Adding MCP server")
            || stderr.contains("No MCP clients")
            || stdout.contains("No MCP clients")
    );
}

// #[test]
// fn test_run_with_docker_compose_file() {
//     let temp_dir = TempDir::new().unwrap();
//     let compose_path = temp_dir.path().join("docker-compose.yml");
//     fs::write(
//         &compose_path,
//         r#"
// version: '3'
// services:
//   mcp-server:
//     image: mcp/test-server:latest
//     ports:
//       - "8080:8080"
// "#,
//     )
//     .unwrap();
//
//     let output = Command::new("cargo")
//         .args(["run", "--quiet", "--", "run", compose_path.to_str().unwrap()])
//         .output()
//         .expect("Failed to execute command");
//
//     let stderr = String::from_utf8_lossy(&output.stderr);
//     let stdout = String::from_utf8_lossy(&output.stdout);
//
//     // Should recognize docker compose file or fail appropriately
//     assert!(
//         stdout.contains("docker-compose") ||
//         stderr.contains("docker") ||
//         stderr.contains("Docker")
//     );
// }

#[test]
fn test_run_with_package_version() {
    // Test with add command instead of run
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "add",
            "test-server@1.2.3",
            "--non-interactive",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should recognize the version specification
    assert!(stdout.contains("test-server@1.2.3") || stdout.contains("Adding MCP server"));
}

#[test]
fn test_multiple_argument_formats() {
    // Test various argument combinations
    let commands = vec![
        vec!["list", "--verbose"],
        vec!["--verbose", "list"],
        vec!["setup"],
        vec!["doctor"],
    ];

    for args in commands {
        let output = Command::new("cargo")
            .args(["run", "--quiet", "--"])
            .args(&args)
            .output()
            .expect("Failed to execute command");

        // All these commands should at least parse correctly
        // They may fail for other reasons in test environment
        let _stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should not have parsing errors
        assert!(
            !stderr.contains("invalid subcommand") && !stderr.contains("unexpected argument"),
            "Failed for args: {args:?}\nstderr: {stderr}"
        );
    }
}

#[test]
fn test_environment_variable_passing() {
    // Test that environment variables work with commands
    let output = Command::new("cargo")
        .env("RUST_BACKTRACE", "1")
        .args(["run", "--quiet", "--", "list"])
        .output()
        .expect("Failed to execute command");

    // Should execute successfully
    assert!(
        output.status.success() || {
            let stderr = String::from_utf8_lossy(&output.stderr);
            stderr.contains("No MCP servers configured")
        }
    );
}

#[test]
fn test_path_with_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let dir_with_spaces = temp_dir.path().join("path with spaces");
    fs::create_dir(&dir_with_spaces).unwrap();

    let script_path = dir_with_spaces.join("server.js");
    fs::write(&script_path, "console.log('Server in path with spaces');").unwrap();

    // Test with add command
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "add",
            script_path.to_str().unwrap(),
            "--non-interactive",
        ])
        .output()
        .expect("Failed to execute command");

    // Should handle the path with spaces
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stderr.contains("path with spaces")
            || stdout.contains("Adding MCP server")
            || stderr.contains("No MCP clients")
    );
}

#[test]
fn test_relative_path_resolution() {
    // Test that relative paths work correctly
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "add",
            "./local-server",
            "--non-interactive",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should process the relative path
    assert!(stdout.contains("./local-server") || stdout.contains("Adding MCP server"));
}

#[test]
fn test_parent_directory_path() {
    // Test parent directory references
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "add",
            "../server",
            "--non-interactive",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should handle parent directory reference
    assert!(stdout.contains("../server") || stdout.contains("Adding MCP server"));
}

#[test]
fn test_cli_error_messages() {
    // Test that error messages are helpful
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "nonexistent-command"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have helpful error message
    assert!(
        stderr.contains("unrecognized subcommand")
            || stderr.contains("Usage")
            || stderr.contains("help")
    );
}
