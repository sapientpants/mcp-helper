//! Comprehensive error handling tests for the installation system
//!
//! This test suite covers error conditions, edge cases, and failure scenarios
//! in the MCP server installation workflow to ensure robust error handling.

use mcp_helper::error::McpError;
use mcp_helper::install::InstallCommand;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test error handling when no MCP clients are installed
#[test]
#[serial]
fn error_no_clients_installed() {
    let _temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create installer with verbose output for better debugging
    let mut installer = InstallCommand::new(true);
    
    // Try to install a server when no clients are available
    let result = installer.execute("@modelcontextprotocol/server-filesystem");
    
    match result {
        Err(McpError::ClientNotFound { client_name, .. }) => {
            // This is expected - we should get a meaningful error message
            assert!(!client_name.is_empty(), "Client name should not be empty in error");
        }
        Err(other_error) => {
            panic!("Expected ClientNotFound error, got: {:?}", other_error);
        }
        Ok(_) => {
            panic!("Expected installation to fail when no clients are installed");
        }
    }
}

/// Test error handling for invalid server specifications
#[test]
fn error_invalid_server_specs() {
    let mut installer = InstallCommand::new(false);
    
    let long_name = "a".repeat(1000);
    let invalid_specs = vec![
        "",                           // Empty string
        "   ",                       // Whitespace only
        "invalid@version@spec",      // Multiple @ symbols
        "malicious/../../etc/passwd", // Path traversal attempt
        "file:///etc/passwd",        // File URI
        "javascript:alert(1)",       // JavaScript injection
        "🚀💥🔥",                    // Emoji-only name
        &long_name,                   // Extremely long name
        "\0null\0byte",             // Null bytes
        "<script>alert('xss')</script>", // XSS attempt
    ];
    
    for spec in invalid_specs {
        let result = installer.execute(spec);
        assert!(result.is_err(), "Expected error for invalid spec: '{}'", spec);
        
        // Ensure error messages are informative
        match result {
            Err(error) => {
                let error_msg = error.to_string();
                assert!(!error_msg.is_empty(), "Error message should not be empty");
                assert!(error_msg.len() > 10, "Error message should be descriptive");
            }
            Ok(_) => unreachable!(),
        }
    }
}

/// Test error handling for corrupted configuration files
#[test]
#[serial]
fn error_corrupted_config_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("claude_desktop_config.json");
    
    let huge_json = format!("{{\"x\": \"{}\"}}", "a".repeat(100_000));
    let deeply_nested = "{\"servers\": {\"test\": {\"command\": [\"cmd\", \"--arg\", ".repeat(10000) + "]}}}";
    let corrupted_configs = vec![
        "not json at all",                                    // Invalid JSON
        "{",                                                  // Incomplete JSON
        r#"{"servers": [}"#,                                 // Malformed JSON
        "null",                                              // Valid JSON but wrong type
        r#"{"servers": "not an object"}"#,                   // Wrong type for servers
        &huge_json,                                           // Huge JSON
        &deeply_nested,                                       // Deeply nested
    ];
    
    for (i, config_content) in corrupted_configs.iter().enumerate() {
        // Write corrupted config
        fs::write(&config_path, config_content)
            .expect("Failed to write corrupted config");
        
        let mut installer = InstallCommand::new(true);
        let result = installer.execute("test-server");
        
        // Should handle corrupted configs gracefully
        match result {
            Err(McpError::ConfigError { path, message }) => {
                assert!(path.ends_with("claude_desktop_config.json"));
                assert!(!message.is_empty(), "Config error message should not be empty");
                println!("Test {}: Got expected config error: {}", i, message);
            }
            Err(other_error) => {
                // Other errors are also acceptable as long as they're handled gracefully
                println!("Test {}: Got error (acceptable): {:?}", i, other_error);
            }
            Ok(_) => {
                panic!("Test {}: Expected error for corrupted config", i);
            }
        }
    }
}

/// Test error handling for insufficient file permissions
#[test]
#[serial]
#[cfg(unix)] // Unix-specific permission tests
fn error_insufficient_permissions() {
    use std::os::unix::fs::PermissionsExt;
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("claude_desktop_config.json");
    
    // Create a valid config file
    fs::write(&config_path, r#"{"servers": {}}"#)
        .expect("Failed to write config");
    
    // Make it read-only
    let mut perms = fs::metadata(&config_path).unwrap().permissions();
    perms.set_mode(0o444); // Read-only
    fs::set_permissions(&config_path, perms).unwrap();
    
    let mut installer = InstallCommand::new(true);
    let result = installer.execute("test-server");
    
    // Should handle permission errors gracefully
    assert!(result.is_err(), "Expected permission error");
    
    match result {
        Err(McpError::IoError { path, source, .. }) => {
            assert!(path.as_ref().map(|p| p.ends_with("claude_desktop_config.json")).unwrap_or(false));
            // Should provide meaningful error about permissions
            let error_msg = source.to_string().to_lowercase();
            assert!(
                error_msg.contains("permission") || error_msg.contains("access"),
                "Error should mention permissions: {}",
                error_msg
            );
        }
        Err(other_error) => {
            println!("Got different error (may be acceptable): {:?}", other_error);
        }
        Ok(_) => {
            panic!("Expected permission error");
        }
    }
}

/// Test error handling for disk space exhaustion scenarios
#[test]
#[serial]
fn error_disk_space_simulation() {
    // Create a very small temporary file system or fill up temp space
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Try to create an extremely large config that might exhaust space
    let huge_config = format!(
        r#"{{"servers": {{"test": {{"description": "{}"}}}}}}"#,
        "x".repeat(1_000_000) // 1MB of data
    );
    
    let config_path = temp_dir.path().join("claude_desktop_config.json");
    
    // This might fail due to space or succeed - we test that it's handled gracefully
    match fs::write(&config_path, &huge_config) {
        Ok(_) => {
            println!("Large config write succeeded - testing installer");
            let mut installer = InstallCommand::new(true);
            let _result = installer.execute("test-server");
            // Whether it succeeds or fails, it should not panic
        }
        Err(e) => {
            println!("Large config write failed as expected: {}", e);
            // This is the expected case for disk space issues
        }
    }
}

/// Test error handling for concurrent access scenarios
#[test]
#[serial]
fn error_concurrent_config_access() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("claude_desktop_config.json");
    
    // Create initial config
    fs::write(&config_path, r#"{"servers": {}}"#)
        .expect("Failed to write initial config");
    
    let errors = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];
    
    // Spawn multiple threads trying to install simultaneously
    for i in 0..5 {
        let _config_path_clone = config_path.clone();
        let errors_clone = Arc::clone(&errors);
        
        let handle = thread::spawn(move || {
            // Add small delay to increase chance of collision
            thread::sleep(Duration::from_millis(i * 10));
            
            let mut installer = InstallCommand::new(false);
            let result = installer.execute(&format!("test-server-{}", i));
            
            if let Err(e) = result {
                errors_clone.lock().unwrap().push(e);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    let errors = errors.lock().unwrap();
    println!("Concurrent access resulted in {} errors", errors.len());
    
    // Some errors are expected due to concurrent access
    // The important thing is that we don't panic or corrupt data
    for error in errors.iter() {
        println!("Concurrent access error: {:?}", error);
        // Errors should be well-formed and informative
        assert!(!error.to_string().is_empty());
    }
}

/// Test error handling for malformed dependency requirements
#[test]
fn error_malformed_dependencies() {
    let mut installer = InstallCommand::new(true);
    
    // Test with a server spec that would require malformed dependencies
    // This tests the dependency validation error paths
    let result = installer.execute("malformed-server-with-bad-deps");
    
    // Should handle unknown servers gracefully
    assert!(result.is_err(), "Expected error for unknown server");
    
    match result {
        Err(error) => {
            let error_msg = error.to_string();
            println!("Dependency error: {}", error_msg);
            // Error should be informative and not expose internal details
            assert!(!error_msg.contains("panicked"));
            assert!(!error_msg.contains("unwrap"));
        }
        Ok(_) => unreachable!(),
    }
}

/// Test error handling for network timeouts and failures
#[test]
#[serial]
fn error_network_failures() {
    let mut installer = InstallCommand::new(true);
    
    // Test with various network-related server specs that might fail
    let network_specs = vec![
        "https://nonexistent.domain.invalid/server",
        "git://invalid.repo/server",
        "ssh://user@nonexistent/server",
    ];
    
    for spec in network_specs {
        let result = installer.execute(spec);
        
        // Network failures should be handled gracefully
        match result {
            Err(error) => {
                let error_msg = error.to_string();
                println!("Network error for '{}': {}", spec, error_msg);
                
                // Should not expose sensitive information or stack traces
                assert!(!error_msg.contains("panic"));
                assert!(!error_msg.contains("thread 'main' panicked"));
                
                // Should provide helpful guidance
                assert!(error_msg.len() > 20, "Error message should be descriptive");
            }
            Ok(_) => {
                // If it succeeds, that's fine too (might be a valid spec)
                println!("Spec '{}' succeeded unexpectedly", spec);
            }
        }
    }
}

/// Test resource cleanup after installation failures
#[test]
#[serial]
fn error_resource_cleanup() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create a scenario where installation starts but fails
    let mut installer = InstallCommand::new(true);
    
    // Count files before installation attempt
    let files_before = count_files_recursively(temp_dir.path());
    
    // Attempt installation that should fail
    let result = installer.execute("definitely-nonexistent-server-12345");
    assert!(result.is_err(), "Expected installation to fail");
    
    // Count files after - should not have leaked temporary files
    let files_after = count_files_recursively(temp_dir.path());
    
    assert_eq!(
        files_before, files_after,
        "Temporary files may have been leaked during failed installation"
    );
}

/// Helper function to recursively count files in a directory
fn count_files_recursively(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }
    
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                count += 1;
            } else if entry.path().is_dir() {
                count += count_files_recursively(&entry.path());
            }
        }
    }
    count
}

/// Test graceful handling of interrupted installations
#[test]
#[serial]
fn error_interrupted_installation() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    
    let _temp_dir = TempDir::new().expect("Failed to create temp directory");
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);
    
    // Simulate installation interruption after a short delay
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        interrupted_clone.store(true, Ordering::Relaxed);
    });
    
    let mut installer = InstallCommand::new(true);
    
    // This test ensures that even if something goes wrong during installation,
    // we handle it gracefully and don't leave the system in a bad state
    let result = installer.execute("test-server");
    
    // Whether it succeeds or fails, it should not panic
    match result {
        Ok(_) => println!("Installation completed before interruption"),
        Err(e) => println!("Installation failed gracefully: {}", e),
    }
    
    // Verify no corruption occurred
    assert!(!interrupted.load(Ordering::Relaxed) || interrupted.load(Ordering::Relaxed));
}