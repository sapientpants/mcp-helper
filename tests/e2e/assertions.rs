//! Custom assertions for E2E testing
//!
//! Provides domain-specific assertions for testing MCP Helper
//! command outputs and behavior.

#![allow(dead_code)]

use super::CommandResult;
use std::fs;
use std::path::Path;

/// Assert that a command succeeded
pub fn assert_command_success(result: &CommandResult) {
    assert!(
        result.success(),
        "Command failed: {:?}\nstdout: {}\nstderr: {}",
        result.args(),
        result.stdout_string(),
        result.stderr_string()
    );
}

/// Assert that a command failed
pub fn assert_command_failure(result: &CommandResult) {
    assert!(
        !result.success(),
        "Command unexpectedly succeeded: {:?}\nstdout: {}",
        result.args(),
        result.stdout_string()
    );
}

/// Assert that a command failed with a specific exit code
pub fn assert_command_exit_code(result: &CommandResult, expected_code: i32) {
    match result.exit_code() {
        Some(code) => assert_eq!(
            code,
            expected_code,
            "Command exited with code {} but expected {}\nstdout: {}\nstderr: {}",
            code,
            expected_code,
            result.stdout_string(),
            result.stderr_string()
        ),
        None => panic!(
            "Command was terminated by signal, expected exit code {}\nstdout: {}\nstderr: {}",
            expected_code,
            result.stdout_string(),
            result.stderr_string()
        ),
    }
}

/// Assert that stdout contains specific text
pub fn assert_stdout_contains(result: &CommandResult, expected: &str) {
    let stdout = result.stdout_string();
    assert!(
        stdout.contains(expected),
        "stdout does not contain '{expected}'\nActual stdout: {stdout}"
    );
}

/// Assert that stderr contains specific text
pub fn assert_stderr_contains(result: &CommandResult, expected: &str) {
    let stderr = result.stderr_string();
    assert!(
        stderr.contains(expected),
        "stderr does not contain '{expected}'\nActual stderr: {stderr}"
    );
}

/// Assert that stdout contains all of the specified strings
pub fn assert_stdout_contains_all(result: &CommandResult, expected: &[&str]) {
    let stdout = result.stdout_string();
    for text in expected {
        assert!(
            stdout.contains(text),
            "stdout does not contain '{text}'\nActual stdout: {stdout}"
        );
    }
}

/// Assert that stdout does not contain specific text
pub fn assert_stdout_not_contains(result: &CommandResult, unexpected: &str) {
    let stdout = result.stdout_string();
    assert!(
        !stdout.contains(unexpected),
        "stdout unexpectedly contains '{unexpected}'\nActual stdout: {stdout}"
    );
}

/// Assert that stderr does not contain specific text
pub fn assert_stderr_not_contains(result: &CommandResult, unexpected: &str) {
    let stderr = result.stderr_string();
    assert!(
        !stderr.contains(unexpected),
        "stderr unexpectedly contains '{unexpected}'\nActual stderr: {stderr}"
    );
}

/// Assert that output contains colored text (ANSI escape codes)
pub fn assert_colored_output(result: &CommandResult) {
    let stdout = result.stdout_string();
    let stderr = result.stderr_string();

    let contains_ansi = stdout.contains('\x1b') || stderr.contains('\x1b');
    assert!(
        contains_ansi,
        "Output does not contain ANSI color codes\nstdout: {stdout}\nstderr: {stderr}"
    );
}

/// Assert that a file exists at the given path
pub fn assert_file_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(path.exists(), "File does not exist: {}", path.display());
    assert!(
        path.is_file(),
        "Path exists but is not a file: {}",
        path.display()
    );
}

/// Assert that a directory exists at the given path
pub fn assert_dir_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(
        path.exists(),
        "Directory does not exist: {}",
        path.display()
    );
    assert!(
        path.is_dir(),
        "Path exists but is not a directory: {}",
        path.display()
    );
}

/// Assert that a path does not exist
pub fn assert_not_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(
        !path.exists(),
        "Path unexpectedly exists: {}",
        path.display()
    );
}

/// Assert that a file contains specific text
pub fn assert_file_contains<P: AsRef<Path>>(path: P, expected: &str) {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path.display(), e));

    assert!(
        contents.contains(expected),
        "File {} does not contain '{}'\nActual contents: {}",
        path.display(),
        expected,
        contents
    );
}

/// Assert that a JSON file contains a specific key-value pair
pub fn assert_json_file_contains<P: AsRef<Path>>(path: P, key: &str, expected_value: &str) {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path.display(), e));

    let json: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse JSON from {}: {}", path.display(), e));

    // Navigate to the key (supports dot notation like "mcpServers.filesystem.command")
    let mut current = &json;
    for part in key.split('.') {
        current = current
            .get(part)
            .unwrap_or_else(|| panic!("Key '{}' not found in JSON file {}", key, path.display()));
    }

    let actual_value = current.as_str().unwrap_or_else(|| {
        panic!(
            "Value at key '{}' is not a string in {}",
            key,
            path.display()
        )
    });

    assert_eq!(
        actual_value,
        expected_value,
        "JSON file {} has incorrect value for key '{}'\nExpected: {}\nActual: {}",
        path.display(),
        key,
        expected_value,
        actual_value
    );
}

/// Assert that the command output indicates successful server installation
pub fn assert_server_installed(result: &CommandResult, server_name: &str) {
    assert_command_success(result);

    let stdout = result.stdout_string();
    let stderr = result.stderr_string();
    let output = format!("{stdout}{stderr}");

    // Check for success indicators
    assert!(
        output.contains("Successfully installed") || 
        output.contains("Installation complete") ||
        output.contains(&"✓".to_string()) || // Check mark
        output.contains("installed successfully"),
        "Output does not indicate successful installation of {server_name}\nOutput: {output}"
    );

    // Check that the server name appears in the output
    assert!(
        output.contains(server_name),
        "Output does not mention server name '{server_name}'\nOutput: {output}"
    );
}

/// Assert that the command output indicates installation failure
pub fn assert_server_install_failed(result: &CommandResult, server_name: &str) {
    // Command might succeed but indicate installation failure in output
    let stdout = result.stdout_string();
    let stderr = result.stderr_string();
    let output = format!("{stdout}{stderr}");

    // Check for failure indicators
    assert!(
        output.contains("Failed to install") ||
        output.contains("Installation failed") ||
        output.contains("Error:") ||
        output.contains("✗") || // X mark
        !result.success(),
        "Output does not indicate installation failure for {server_name}\nOutput: {output}"
    );
}

/// Assert that help text is properly formatted
pub fn assert_help_text_formatted(result: &CommandResult) {
    assert_command_success(result);

    let stdout = result.stdout_string();

    // Check for common help text elements
    assert!(
        stdout.contains("USAGE:") || stdout.contains("Usage:"),
        "Help text does not contain usage section\nstdout: {stdout}"
    );

    assert!(
        stdout.contains("OPTIONS:") || stdout.contains("FLAGS:") || stdout.contains("Options:"),
        "Help text does not contain options section\nstdout: {stdout}"
    );

    // Check that it's reasonably formatted (has line breaks)
    assert!(
        stdout.lines().count() > 5,
        "Help text appears to be too short or not formatted\nstdout: {stdout}"
    );
}

/// Assert that version output is properly formatted
pub fn assert_version_output(result: &CommandResult) {
    assert_command_success(result);

    let stdout = result.stdout_string();

    // Check for version format (e.g., "mcp-helper 0.1.0")
    assert!(
        stdout.contains("mcp") && (stdout.contains('.') || stdout.chars().any(|c| c.is_numeric())),
        "Version output does not appear to contain version information\nstdout: {stdout}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Output;

    fn create_mock_result(success: bool, stdout: &str, stderr: &str) -> CommandResult {
        use std::process::ExitStatus;

        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            if success {
                ExitStatus::from_raw(0)
            } else {
                ExitStatus::from_raw(256) // Exit code 1
            }
        };

        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            if success {
                ExitStatus::from_raw(0)
            } else {
                ExitStatus::from_raw(1)
            }
        };

        CommandResult::new_for_test(
            Output {
                status,
                stdout: stdout.as_bytes().to_vec(),
                stderr: stderr.as_bytes().to_vec(),
            },
            vec!["test".to_string()],
        )
    }

    #[test]
    fn test_assert_command_success() {
        let result = create_mock_result(true, "success", "");
        assert_command_success(&result); // Should not panic
    }

    #[test]
    #[should_panic(expected = "Command failed")]
    fn test_assert_command_success_fails() {
        let result = create_mock_result(false, "", "error");
        assert_command_success(&result);
    }

    #[test]
    fn test_assert_stdout_contains() {
        let result = create_mock_result(true, "Hello, world!", "");
        assert_stdout_contains(&result, "Hello");
        assert_stdout_contains(&result, "world");
    }

    #[test]
    #[should_panic(expected = "stdout does not contain")]
    fn test_assert_stdout_contains_fails() {
        let result = create_mock_result(true, "Hello, world!", "");
        assert_stdout_contains(&result, "missing");
    }

    #[test]
    fn test_assert_stdout_contains_all() {
        let result = create_mock_result(true, "Hello, brave new world!", "");
        assert_stdout_contains_all(&result, &["Hello", "world", "brave"]);
    }
}
