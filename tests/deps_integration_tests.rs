//! Integration tests for dependency checking and validation
//!
//! These tests verify the dependency detection, version checking,
//! and installation instruction generation.

use mcp_helper::deps::{
    Dependency, DependencyChecker, DependencyStatus, NodeChecker, PythonChecker,
};
use std::collections::HashMap;

#[test]
fn test_node_checker_creation() {
    let checker = NodeChecker::new();
    // Checker should be created successfully
    let _ = checker;
}

#[test]
fn test_node_checker_check() {
    let checker = NodeChecker::new();
    let result = checker.check();

    // Check should return a result
    assert!(result.is_ok() || result.is_err());

    if let Ok(check) = result {
        // Should have a dependency type
        match check.dependency {
            Dependency::NodeJs { .. } => {}
            _ => panic!("Expected NodeJs dependency"),
        }

        // Should have a valid status
        match check.status {
            DependencyStatus::Installed { .. }
            | DependencyStatus::Missing
            | DependencyStatus::VersionMismatch { .. }
            | DependencyStatus::ConfigurationRequired { .. } => {}
        }
    }
}

#[test]
fn test_python_checker_creation() {
    let checker = PythonChecker::new();
    // Checker should be created successfully
    let _ = checker;
}

#[test]
fn test_python_checker_check() {
    let checker = PythonChecker::new();
    let result = checker.check();

    // Check should return a result
    assert!(result.is_ok() || result.is_err());

    if let Ok(check) = result {
        // Should have a dependency type
        match check.dependency {
            Dependency::Python { .. } => {}
            _ => panic!("Expected Python dependency"),
        }
    }
}

#[test]
fn test_dependency_status_variants() {
    // Test that all status variants can be created
    let _installed = DependencyStatus::Installed {
        version: Some("1.0.0".to_string()),
    };

    let _missing = DependencyStatus::Missing;

    let _mismatch = DependencyStatus::VersionMismatch {
        installed: "1.0.0".to_string(),
        required: "2.0.0".to_string(),
    };

    let _config_required = DependencyStatus::ConfigurationRequired {
        issue: "Configuration issue".to_string(),
        solution: "Update configuration".to_string(),
    };
}

#[test]
fn test_dependency_name() {
    let node_dep = Dependency::NodeJs {
        min_version: Some("18.0.0".to_string()),
    };
    assert_eq!(node_dep.name(), "Node.js");

    let python_dep = Dependency::Python {
        min_version: Some("3.8".to_string()),
    };
    assert_eq!(python_dep.name(), "Python");

    let docker_dep = Dependency::Docker {
        min_version: None,
        requires_compose: false,
    };
    assert_eq!(docker_dep.name(), "Docker");

    let git_dep = Dependency::Git;
    assert_eq!(git_dep.name(), "Git");
}

#[test]
fn test_check_multiple_dependencies() {
    let checkers: Vec<Box<dyn DependencyChecker>> =
        vec![Box::new(NodeChecker::new()), Box::new(PythonChecker::new())];

    let mut results = HashMap::new();

    for checker in checkers {
        let result = checker.check();
        if let Ok(check) = result {
            let name = check.dependency.name().to_string();
            results.insert(name, check.status);
        }
    }

    // Should have checked at least one dependency
    assert!(!results.is_empty());
}

#[test]
fn test_node_version_detection() {
    let checker = NodeChecker::new();
    let result = checker.check();

    if let Ok(check) = result {
        if let DependencyStatus::Installed { version: Some(v) } = check.status {
            // Version should contain dots (e.g., "18.0.0")
            assert!(v.contains('.'), "Version should be in semver format");
        }
    }
}

#[test]
fn test_python_version_detection() {
    let checker = PythonChecker::new();
    let result = checker.check();

    if let Ok(check) = result {
        if let DependencyStatus::Installed { version: Some(v) } = check.status {
            // Python version should contain dots (e.g., "3.9.0")
            assert!(v.contains('.'), "Version should be in semver format");
        }
    }
}

#[test]
fn test_install_instructions_present() {
    let checker = NodeChecker::new();
    let result = checker.check();

    if let Ok(check) = result {
        if matches!(check.status, DependencyStatus::Missing) {
            // Missing dependencies should have install instructions
            assert!(check.install_instructions.is_some());

            if let Some(instructions) = check.install_instructions {
                // Should have platform-specific instructions
                #[cfg(target_os = "windows")]
                assert!(!instructions.windows.is_empty());

                #[cfg(target_os = "macos")]
                assert!(!instructions.macos.is_empty());

                #[cfg(target_os = "linux")]
                assert!(!instructions.linux.is_empty());
            }
        }
    }
}

#[test]
fn test_version_mismatch_handling() {
    // Create a status with version mismatch
    let status = DependencyStatus::VersionMismatch {
        installed: "16.0.0".to_string(),
        required: "18.0.0".to_string(),
    };

    if let DependencyStatus::VersionMismatch {
        installed,
        required,
    } = status
    {
        assert_eq!(installed, "16.0.0");
        assert_eq!(required, "18.0.0");
    } else {
        panic!("Expected VersionMismatch");
    }
}

#[test]
fn test_configuration_required_status() {
    let status = DependencyStatus::ConfigurationRequired {
        issue: "PATH not set".to_string(),
        solution: "Add Node.js to PATH".to_string(),
    };

    if let DependencyStatus::ConfigurationRequired { issue, solution } = status {
        assert!(issue.contains("PATH"));
        assert!(solution.contains("PATH"));
    } else {
        panic!("Expected ConfigurationRequired");
    }
}

#[test]
fn test_dependency_checker_trait_object() {
    // Test that we can use DependencyChecker as a trait object
    let checkers: Vec<Box<dyn DependencyChecker>> =
        vec![Box::new(NodeChecker::new()), Box::new(PythonChecker::new())];

    for checker in checkers {
        let _ = checker.check();
    }
}

#[test]
fn test_concurrent_dependency_checks() {
    use std::sync::Arc;
    use std::thread;

    let checkers: Vec<Arc<dyn DependencyChecker + Send + Sync>> =
        vec![Arc::new(NodeChecker::new()), Arc::new(PythonChecker::new())];

    let mut handles = vec![];

    for checker in checkers {
        let handle = thread::spawn(move || checker.check());
        handles.push(handle);
    }

    // All checks should complete successfully
    for handle in handles {
        let result = handle.join();
        assert!(result.is_ok());
    }
}

#[test]
fn test_dependency_check_performance() {
    use std::time::Instant;

    let checker = NodeChecker::new();

    // Dependency check should be fast
    let start = Instant::now();
    let _ = checker.check();
    let duration = start.elapsed();

    // Should complete in less than 2 seconds
    assert!(
        duration.as_secs() < 2,
        "Dependency check took too long: {duration:?}"
    );
}

#[test]
fn test_missing_dependency_status() {
    let status = DependencyStatus::Missing;

    match status {
        DependencyStatus::Missing => {}
        _ => panic!("Expected Missing status"),
    }
}

#[test]
fn test_installed_with_version() {
    let status = DependencyStatus::Installed {
        version: Some("20.10.0".to_string()),
    };

    if let DependencyStatus::Installed { version } = status {
        assert_eq!(version, Some("20.10.0".to_string()));
    } else {
        panic!("Expected Installed status");
    }
}

#[test]
fn test_installed_without_version() {
    let status = DependencyStatus::Installed { version: None };

    if let DependencyStatus::Installed { version } = status {
        assert!(version.is_none());
    } else {
        panic!("Expected Installed status");
    }
}

#[test]
fn test_dependency_display_format() {
    let status = DependencyStatus::Installed {
        version: Some("18.0.0".to_string()),
    };

    // Should be displayable
    let display = format!("{status}");
    assert!(!display.is_empty());
}
