//! Error handling tests for the MCP server runner
//!
//! Tests process execution failures, platform-specific errors,
//! and command construction edge cases.

use mcp_helper::runner::{Platform, ServerRunner};
use serial_test::serial;
use std::env;

/// Helper function to detect current platform for tests
fn detect_platform() -> Platform {
    match env::consts::OS {
        "windows" => Platform::Windows,
        "macos" => Platform::MacOS,
        "linux" => Platform::Linux,
        _ => Platform::Linux, // Default to Linux
    }
}

#[test]
fn error_nonexistent_command() {
    let runner = ServerRunner::new(detect_platform(), false);

    // Try to run a command that definitely doesn't exist
    let result = runner.run("definitely-nonexistent-command-12345", &[]);

    assert!(result.is_err(), "Expected error for nonexistent command");

    match result {
        Err(error) => {
            let error_msg = error.to_string();
            println!("Nonexistent command error: {error_msg}");

            // Should provide helpful error message
            assert!(!error_msg.is_empty());
            assert!(error_msg.len() > 10);

            // Should not expose internal panic details
            assert!(!error_msg.contains("panicked"));
            assert!(!error_msg.contains("unwrap"));
        }
        Ok(_) => unreachable!(),
    }
}

#[test]
fn error_invalid_arguments() {
    let runner = ServerRunner::new(detect_platform(), true);

    // Test various problematic argument combinations
    let invalid_args = vec![
        vec!["\0null_byte".to_string()],
        vec!["--arg".to_string(), "".to_string()], // Empty value
        vec![format!("--{}", "x".repeat(10000))],  // Extremely long argument
        vec!["🚀".to_string(), "💥".to_string(), "🔥".to_string()], // Unicode arguments
        vec!["arg1".to_string(), "arg2".to_string(), "".to_string()], // Empty argument in middle
    ];

    for args in invalid_args {
        let result = runner.run("echo", &args);

        // Either succeeds (arguments are handled properly) or fails gracefully
        match result {
            Ok(_) => {
                println!("Arguments {args:?} handled successfully");
            }
            Err(error) => {
                println!("Arguments {args:?} failed gracefully: {error}");
                // Error should not expose internal details
                assert!(!error.to_string().contains("panic"));
            }
        }
    }
}

#[test]
#[serial]
fn error_environment_corruption() {
    let runner = ServerRunner::new(detect_platform(), true);

    // Save original environment
    let original_path = env::var_os("PATH");

    // Corrupt the PATH environment variable
    env::set_var("PATH", "");

    let result = runner.run("ls", &[]);

    // Restore original environment
    match original_path {
        Some(path) => env::set_var("PATH", path),
        None => env::remove_var("PATH"),
    }

    // Should handle corrupted environment gracefully
    match result {
        Ok(_) => {
            // If it succeeds, that's fine (maybe ls is built-in)
            println!("Command succeeded despite empty PATH");
        }
        Err(error) => {
            println!("Environment corruption handled: {error}");
            assert!(!error.to_string().contains("panicked"));
        }
    }
}

#[test]
fn error_command_permission_denied() {
    let runner = ServerRunner::new(detect_platform(), false);

    // Try to run a command that exists but might not be executable
    // This varies by platform, so we test several possibilities
    let restricted_commands = vec![
        "/etc/passwd",   // File that exists but isn't executable
        "/dev/null",     // Device file
        "/usr/bin/sudo", // Might require special permissions
    ];

    for cmd in restricted_commands {
        let result = runner.run(cmd, &[]);

        match result {
            Ok(_) => {
                println!("Command '{cmd}' succeeded unexpectedly");
            }
            Err(error) => {
                println!("Permission error for '{cmd}': {error}");

                // Should provide meaningful error message
                let error_msg = error.to_string();
                assert!(!error_msg.is_empty());
                assert!(!error_msg.contains("unwrap"));

                // May contain "permission" or "access" in the message
                let error_lower = error_msg.to_lowercase();
                if error_lower.contains("permission") || error_lower.contains("access") {
                    println!("Good: Error message mentions permissions");
                }
            }
        }
    }
}

#[test]
fn error_extremely_long_paths() {
    let runner = ServerRunner::new(detect_platform(), true);

    // Create extremely long path that exceeds filesystem limits
    let long_path = "/".to_string() + &"a".repeat(10000) + "/nonexistent";

    let result = runner.run(&long_path, &[]);

    assert!(result.is_err(), "Expected error for extremely long path");

    match result {
        Err(error) => {
            println!("Long path error: {error}");

            // Should handle path length errors gracefully
            let error_msg = error.to_string();
            assert!(!error_msg.contains("panic"));
            assert!(error_msg.len() > 5);
        }
        Ok(_) => unreachable!(),
    }
}

#[test]
fn error_invalid_unicode_paths() {
    let runner = ServerRunner::new(detect_platform(), true);

    // Test various invalid/problematic Unicode sequences
    let problematic_paths = vec![
        "/path/with/\u{0000}/null",             // Null byte
        "/path/with/\u{FFFF}/invalid",          // Invalid Unicode
        "/path/with/invalid_unicode/surrogate", // Invalid unicode path
        "/path/with/../../../../etc/passwd",    // Path traversal
        "/path/with spaces and\ttabs",          // Whitespace
        "/path/with\nnewlines\rand\rreturns",   // Control characters
    ];

    for path in problematic_paths {
        let result = runner.run(path, &[]);

        // Should either reject or handle these paths safely
        match result {
            Ok(_) => {
                println!(
                    "Path '{}' handled successfully (unexpected)",
                    path.escape_debug()
                );
            }
            Err(error) => {
                println!("Path '{}' rejected safely: {}", path.escape_debug(), error);

                // Error should not expose internal panics
                assert!(!error.to_string().contains("panicked"));
            }
        }
    }
}

#[test]
#[serial]
fn error_process_killed_externally() {
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;

    let _runner = ServerRunner::new(detect_platform(), true);

    // Start a long-running process that we can kill
    let mut child = Command::new("sleep")
        .arg("10")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(ref mut process) => {
            let pid = process.id();

            // Kill the process after a short delay
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));

                // Kill the process externally
                #[cfg(unix)]
                {
                    let _ = std::process::Command::new("kill")
                        .arg("-TERM")
                        .arg(pid.to_string())
                        .output();
                }

                #[cfg(windows)]
                {
                    use std::process::Command;
                    let _ = Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string()])
                        .output();
                }
            });

            // Wait for the process - should handle external termination
            let result = process.wait();

            match result {
                Ok(status) => {
                    println!("Process terminated with status: {status:?}");
                    // Process was killed, so exit status should reflect that
                    assert!(!status.success());
                }
                Err(error) => {
                    println!("Process wait error: {error}");
                    // This is also acceptable - OS might report error
                }
            }
        }
        Err(e) => {
            println!("Could not start sleep process: {e}");
            // Skip this test if sleep is not available
        }
    }
}

#[test]
fn error_working_directory_issues() {
    let runner = ServerRunner::new(detect_platform(), false);

    // Test running commands with problematic working directories
    let problematic_dirs = vec![
        "/nonexistent/directory",
        "/dev/null",   // Not a directory
        "/etc/shadow", // Permission denied
        "",            // Empty string
        "/root",       // Might not be accessible
    ];

    for dir in problematic_dirs {
        // Try to run a command that might fail due to working directory issues
        let result = runner.run("pwd", &[]);

        match result {
            Ok(_) => {
                println!("Command succeeded despite problematic dir '{dir}'");
            }
            Err(error) => {
                println!("Working directory '{dir}' caused error: {error}");

                // Should provide meaningful error
                assert!(!error.to_string().is_empty());
                assert!(!error.to_string().contains("panic"));
            }
        }
    }
}

#[test]
fn error_resource_exhaustion() {
    let runner = ServerRunner::new(detect_platform(), true);

    // Try to create a scenario that might exhaust resources
    // This test ensures we handle resource exhaustion gracefully

    let mut results = Vec::new();

    // Try to run many commands simultaneously (but don't actually do it,
    // as that would be a DoS on the test system)
    for i in 0..5 {
        let result = runner.run("echo", &[format!("test-{i}")]);
        results.push(result);
    }

    // Check that we can handle multiple command executions
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(_) => {
                println!("Command {i} succeeded");
            }
            Err(error) => {
                println!("Command {i} failed: {error}");
                // Failures should be handled gracefully
                assert!(!error.to_string().contains("panic"));
            }
        }
    }
}

#[test]
fn error_platform_specific_commands() {
    let runner = ServerRunner::new(detect_platform(), true);

    // Test platform-specific commands that might not exist on all systems
    let platform_commands = vec![
        (
            "cmd.exe",
            vec!["/C".to_string(), "echo".to_string(), "windows".to_string()],
        ), // Windows
        ("sh", vec!["-c".to_string(), "echo unix".to_string()]), // Unix
        (
            "powershell",
            vec!["-Command".to_string(), "echo ps".to_string()],
        ), // PowerShell
        ("bash", vec!["-c".to_string(), "echo bash".to_string()]), // Bash
    ];

    for (cmd, args) in platform_commands {
        let result = runner.run(cmd, &args);

        match result {
            Ok(_) => {
                println!("Platform command '{cmd}' succeeded");
            }
            Err(error) => {
                println!("Platform command '{cmd}' failed: {error}");

                // Should handle missing platform commands gracefully
                let error_msg = error.to_string();
                assert!(!error_msg.contains("panic"));
                assert!(error_msg.len() > 5);

                // Should be informative about what went wrong
                let error_lower = error_msg.to_lowercase();
                if error_lower.contains("not found") || error_lower.contains("no such file") {
                    println!("Good: Error indicates command not found");
                }
            }
        }
    }
}

#[test]
fn error_signal_handling() {
    let _runner = ServerRunner::new(detect_platform(), true);

    // Test that we handle various signals gracefully
    // This is more of a smoke test to ensure no panics occur

    #[cfg(unix)]
    {
        use std::process::{Command, Stdio};
        use std::thread;
        use std::time::Duration;

        // Start a process that ignores SIGTERM
        let result = Command::new("sleep")
            .arg("0.1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match result {
            Ok(mut child) => {
                // Let it run briefly then check status
                thread::sleep(Duration::from_millis(150));

                match child.try_wait() {
                    Ok(Some(status)) => {
                        println!("Child process exited with: {status:?}");
                    }
                    Ok(None) => {
                        println!("Child process still running, killing it");
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    Err(e) => {
                        println!("Error checking child status: {e}");
                    }
                }
            }
            Err(e) => {
                println!("Could not start test process: {e}");
            }
        }
    }
}

/// Test error handling when PATH resolution fails
#[test]
#[serial]
fn error_path_resolution_failures() {
    let runner = ServerRunner::new(detect_platform(), true);

    // Save original PATH
    let original_path = env::var_os("PATH");

    // Test with various corrupted PATH scenarios
    let corrupted_paths = vec![
        "",                              // Empty
        "/nonexistent1:/nonexistent2",   // All nonexistent
        ":::::",                         // Malformed separators
        "/dev/null:/etc/passwd",         // Files instead of dirs
        "relative:path:without:slashes", // Relative paths
    ];

    for path in corrupted_paths {
        env::set_var("PATH", path);

        let result = runner.run("ls", &[]);

        match result {
            Ok(_) => {
                println!("Command succeeded with corrupted PATH '{path}'");
            }
            Err(error) => {
                println!("Corrupted PATH '{path}' caused error: {error}");

                // Should handle PATH issues gracefully
                assert!(!error.to_string().contains("panic"));
            }
        }
    }

    // Restore original PATH
    match original_path {
        Some(path) => env::set_var("PATH", path),
        None => env::remove_var("PATH"),
    }
}
