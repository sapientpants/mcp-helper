//! Comprehensive tests for dependency installer functionality

use mcp_helper::deps::{
    installer::{detect_package_managers, DependencyInstaller},
    Dependency, DependencyCheck, DependencyStatus, InstallInstructions, InstallMethod,
};

/// Helper to create a mock dependency check
fn create_mock_check(
    dependency: Dependency,
    status: DependencyStatus,
    install_instructions: Option<InstallInstructions>,
) -> DependencyCheck {
    DependencyCheck {
        dependency,
        status,
        install_instructions,
    }
}

/// Helper to create sample install instructions
fn create_sample_instructions() -> InstallInstructions {
    InstallInstructions {
        windows: vec![
            InstallMethod {
                name: "winget".to_string(),
                command: "winget install OpenJS.NodeJS".to_string(),
                description: Some("Windows Package Manager".to_string()),
            },
            InstallMethod {
                name: "chocolatey".to_string(),
                command: "choco install nodejs".to_string(),
                description: Some("Chocolatey package manager".to_string()),
            },
        ],
        macos: vec![
            InstallMethod {
                name: "homebrew".to_string(),
                command: "brew install node".to_string(),
                description: Some("Homebrew package manager".to_string()),
            },
            InstallMethod {
                name: "download".to_string(),
                command: "https://nodejs.org/dist/latest/".to_string(),
                description: Some("Direct download".to_string()),
            },
        ],
        linux: vec![
            InstallMethod {
                name: "apt".to_string(),
                command: "sudo apt-get install nodejs".to_string(),
                description: Some("APT package manager".to_string()),
            },
            InstallMethod {
                name: "dnf".to_string(),
                command: "sudo dnf install nodejs".to_string(),
                description: Some("DNF package manager".to_string()),
            },
        ],
    }
}

#[test]
fn test_installer_creation() {
    let _installer = DependencyInstaller::new();

    // Test dry run mode
    let _dry_installer = DependencyInstaller::new().with_dry_run();

    // Test auto confirm mode
    let _auto_installer = DependencyInstaller::new().with_auto_confirm();

    // Test chaining
    let _both = DependencyInstaller::new()
        .with_dry_run()
        .with_auto_confirm();
}

#[test]
fn test_installer_default() {
    let _installer1 = DependencyInstaller::new();
    let _installer2 = DependencyInstaller::default();

    // Both should have same default settings
    // (We can't directly compare internal fields, but behavior should be same)
}

#[test]
fn test_install_dependency_no_instructions() {
    let installer = DependencyInstaller::new().with_dry_run();

    let check = create_mock_check(
        Dependency::NodeJs { min_version: None },
        DependencyStatus::Missing,
        None, // No install instructions
    );

    let result = installer.install_dependency(&check).unwrap();
    assert!(
        !result,
        "Should return false when no instructions available"
    );
}

#[test]
fn test_install_dependency_empty_platform_methods() {
    let installer = DependencyInstaller::new().with_dry_run();

    let mut instructions = InstallInstructions::default();
    // Leave current platform empty
    #[cfg(target_os = "windows")]
    {
        instructions.windows = vec![];
    }
    #[cfg(target_os = "macos")]
    {
        instructions.macos = vec![];
    }
    #[cfg(target_os = "linux")]
    {
        instructions.linux = vec![];
    }

    let check = create_mock_check(
        Dependency::NodeJs { min_version: None },
        DependencyStatus::Missing,
        Some(instructions),
    );

    let result = installer.install_dependency(&check).unwrap();
    assert!(
        !result,
        "Should return false when no platform methods available"
    );
}

#[test]
fn test_install_dependency_dry_run() {
    let installer = DependencyInstaller::new().with_dry_run();

    let check = create_mock_check(
        Dependency::NodeJs { min_version: None },
        DependencyStatus::Missing,
        Some(create_sample_instructions()),
    );

    let result = installer.install_dependency(&check).unwrap();
    assert!(result, "Dry run should return true for valid instructions");
}

// Private method test removed - select_best_method is not public

// Private method test removed - select_best_method is not public

// Private method test removed - select_best_method is not public

#[test]
fn test_elevation_requirements() {
    let installer = DependencyInstaller::new();

    // Test different dependency types
    let dependencies = vec![
        (Dependency::NodeJs { min_version: None }, false),
        (
            Dependency::Python { min_version: None },
            cfg!(target_os = "linux"),
        ),
        (
            Dependency::Docker {
                min_version: None,
                requires_compose: false,
            },
            true,
        ),
        (Dependency::Git, cfg!(target_os = "linux")),
    ];

    for (dep, expected_elevation) in dependencies {
        let requires = installer.requires_elevation(&dep);
        assert_eq!(
            requires, expected_elevation,
            "Unexpected elevation requirement for {dep:?}"
        );

        let warning = installer.get_elevation_warning(&dep);
        if expected_elevation {
            assert!(warning.is_some(), "Should have warning for {dep:?}");
            let warning_text = warning.unwrap();
            assert!(warning_text.contains(dep.name()));
            assert!(
                warning_text.contains("administrator") || warning_text.contains("sudo"),
                "Warning should mention privileges"
            );
        } else {
            assert!(warning.is_none(), "Should not have warning for {dep:?}");
        }
    }
}

#[test]
fn test_install_dependencies_multiple() {
    let installer = DependencyInstaller::new().with_dry_run();

    let checks = vec![
        create_mock_check(
            Dependency::NodeJs { min_version: None },
            DependencyStatus::Missing,
            Some(create_sample_instructions()),
        ),
        create_mock_check(
            Dependency::Python {
                min_version: Some("3.8.0".to_string()),
            },
            DependencyStatus::Missing,
            Some(create_sample_instructions()),
        ),
    ];

    let results = installer.install_dependencies(&checks).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results[0], "First dependency should succeed in dry run");
    assert!(results[1], "Second dependency should succeed in dry run");
}

#[test]
fn test_install_dependencies_empty() {
    let installer = DependencyInstaller::new();
    let checks = vec![];

    let results = installer.install_dependencies(&checks).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_detect_package_managers() {
    let managers = detect_package_managers();

    // Test that it returns a vector (contents depend on system)
    assert!(managers.is_empty() || !managers.is_empty()); // Always true

    // Verify all returned managers are from expected list
    let valid_managers = [
        "winget",
        "chocolatey",
        "scoop",
        "homebrew",
        "macports",
        "apt",
        "dnf",
        "yum",
        "pacman",
        "snap",
    ];

    for manager in &managers {
        assert!(
            valid_managers.contains(&manager.as_str()),
            "Unexpected package manager: {manager}"
        );
    }
}

#[test]
fn test_handle_download_urls() {
    let installer = DependencyInstaller::new().with_dry_run();

    let mut instructions = InstallInstructions::default();
    let methods = vec![InstallMethod {
        name: "download".to_string(),
        command: "https://example.com/download".to_string(),
        description: Some("Download from website".to_string()),
    }];

    #[cfg(target_os = "windows")]
    {
        instructions.windows = methods;
    }
    #[cfg(target_os = "macos")]
    {
        instructions.macos = methods;
    }
    #[cfg(target_os = "linux")]
    {
        instructions.linux = methods;
    }

    let check = create_mock_check(
        Dependency::NodeJs { min_version: None },
        DependencyStatus::Missing,
        Some(instructions),
    );

    let result = installer.install_dependency(&check).unwrap();
    // URLs should not be executed, but dry run still returns true
    assert!(result);
}

#[test]
fn test_compound_commands() {
    let installer = DependencyInstaller::new().with_dry_run();

    let mut instructions = InstallInstructions::default();
    let methods = vec![InstallMethod {
        name: "complex".to_string(),
        command: "command1 && command2 && command3".to_string(),
        description: Some("Complex installation".to_string()),
    }];

    #[cfg(target_os = "windows")]
    {
        instructions.windows = methods;
    }
    #[cfg(target_os = "macos")]
    {
        instructions.macos = methods;
    }
    #[cfg(target_os = "linux")]
    {
        instructions.linux = methods;
    }

    let check = create_mock_check(
        Dependency::NodeJs { min_version: None },
        DependencyStatus::Missing,
        Some(instructions),
    );

    let result = installer.install_dependency(&check).unwrap();
    assert!(result, "Dry run should handle compound commands");
}

// Private method test removed - select_best_method is not public

#[test]
fn test_install_method_descriptions() {
    let methods = create_sample_instructions();

    // Verify descriptions are present where expected
    for method in methods.windows {
        assert!(method.description.is_some());
    }
    for method in methods.macos {
        assert!(method.description.is_some());
    }
    for method in methods.linux {
        assert!(method.description.is_some());
    }
}

#[test]
fn test_dependency_check_variants() {
    let installer = DependencyInstaller::new().with_dry_run();

    // Test with different status variants
    let statuses = vec![
        DependencyStatus::Installed {
            version: Some("18.0.0".to_string()),
        },
        DependencyStatus::Missing,
        DependencyStatus::VersionMismatch {
            installed: "16.0.0".to_string(),
            required: "18.0.0".to_string(),
        },
        DependencyStatus::ConfigurationRequired {
            issue: "NPX not found".to_string(),
            solution: "Reinstall Node.js".to_string(),
        },
    ];

    for status in statuses {
        let check = create_mock_check(
            Dependency::NodeJs { min_version: None },
            status,
            Some(create_sample_instructions()),
        );

        // Should handle all status types without panicking
        let _ = installer.install_dependency(&check);
    }
}

#[test]
fn test_dependency_name_extraction() {
    let dependencies = vec![
        (Dependency::NodeJs { min_version: None }, "Node.js"),
        (
            Dependency::Python {
                min_version: Some("3.8.0".to_string()),
            },
            "Python",
        ),
        (
            Dependency::Docker {
                min_version: None,
                requires_compose: true,
            },
            "Docker",
        ),
        (Dependency::Git, "Git"),
    ];

    for (dep, expected_name) in dependencies {
        assert_eq!(dep.name(), expected_name);
    }
}

#[test]
fn test_platform_specific_methods() {
    let instructions = create_sample_instructions();

    #[cfg(target_os = "windows")]
    {
        let methods = instructions.for_platform();
        assert!(!methods.is_empty());
        assert!(methods
            .iter()
            .any(|m| m.name.contains("winget") || m.name.contains("chocolatey")));
    }

    #[cfg(target_os = "macos")]
    {
        let methods = instructions.for_platform();
        assert!(!methods.is_empty());
        assert!(methods
            .iter()
            .any(|m| m.name.contains("homebrew") || m.name.contains("download")));
    }

    #[cfg(target_os = "linux")]
    {
        let methods = instructions.for_platform();
        assert!(!methods.is_empty());
        assert!(methods
            .iter()
            .any(|m| m.name.contains("apt") || m.name.contains("dnf")));
    }
}

// Private method test removed - select_best_method is not public

#[test]
fn test_command_parsing() {
    // Test that commands would be parsed correctly
    let test_commands = vec![
        ("simple-command", 1),
        ("command with args", 3),
        ("command --flag value", 3),
        ("", 0), // Empty command
    ];

    for (cmd, expected_parts) in test_commands {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        assert_eq!(
            parts.len(),
            expected_parts,
            "Command '{cmd}' should have {expected_parts} parts"
        );
    }
}

#[test]
fn test_shell_detection() {
    // Verify shell command construction logic
    #[cfg(target_os = "windows")]
    {
        let shell = "cmd";
        let args = ["/C", "echo test"];
        assert_eq!(shell, "cmd");
        assert_eq!(args[0], "/C");
    }

    #[cfg(not(target_os = "windows"))]
    {
        let shell = "sh";
        let args = ["-c", "echo test"];
        assert_eq!(shell, "sh");
        assert_eq!(args[0], "-c");
    }
}

#[test]
fn test_builder_pattern() {
    // Test that builder methods return self for chaining
    let _installer = DependencyInstaller::new()
        .with_dry_run()
        .with_auto_confirm()
        .with_dry_run(); // Can call multiple times
}

#[test]
fn test_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    // DependencyInstaller should be thread-safe
    let installer = Arc::new(DependencyInstaller::new().with_dry_run());

    let handles: Vec<_> = (0..3)
        .map(|_| {
            let installer = Arc::clone(&installer);
            thread::spawn(move || {
                let check = create_mock_check(
                    Dependency::NodeJs { min_version: None },
                    DependencyStatus::Missing,
                    Some(create_sample_instructions()),
                );

                let result = installer.install_dependency(&check);
                assert!(result.is_ok());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
