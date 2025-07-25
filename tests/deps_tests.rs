use mcp_helper::deps::{
    get_install_instructions, Dependency, DependencyCheck, DependencyChecker, DependencyStatus,
    InstallInstructions, InstallMethod,
};

#[test]
fn test_dependency_name() {
    assert_eq!(Dependency::NodeJs { min_version: None }.name(), "Node.js");
    assert_eq!(
        Dependency::NodeJs {
            min_version: Some("18.0.0".to_string())
        }
        .name(),
        "Node.js"
    );
    assert_eq!(Dependency::Python { min_version: None }.name(), "Python");
    assert_eq!(Dependency::Docker.name(), "Docker");
    assert_eq!(Dependency::Git.name(), "Git");
}

#[test]
fn test_dependency_status_display() {
    let installed = DependencyStatus::Installed { version: None };
    assert_eq!(format!("{installed}"), "Installed");

    let installed_with_version = DependencyStatus::Installed {
        version: Some("18.0.0".to_string()),
    };
    assert_eq!(format!("{installed_with_version}"), "Installed (18.0.0)");

    let missing = DependencyStatus::Missing;
    assert_eq!(format!("{missing}"), "Not installed");

    let mismatch = DependencyStatus::VersionMismatch {
        installed: "16.0.0".to_string(),
        required: "18.0.0".to_string(),
    };
    assert_eq!(
        format!("{mismatch}"),
        "Version mismatch (installed: 16.0.0, required: 18.0.0)"
    );
}

#[test]
fn test_install_instructions_for_platform() {
    let instructions = InstallInstructions {
        windows: vec![InstallMethod {
            name: "winget".to_string(),
            command: "winget install".to_string(),
            description: None,
        }],
        macos: vec![InstallMethod {
            name: "brew".to_string(),
            command: "brew install".to_string(),
            description: None,
        }],
        linux: vec![InstallMethod {
            name: "apt".to_string(),
            command: "apt install".to_string(),
            description: None,
        }],
    };

    #[cfg(target_os = "windows")]
    assert_eq!(instructions.for_platform().len(), 1);

    #[cfg(target_os = "macos")]
    assert_eq!(instructions.for_platform().len(), 1);

    #[cfg(target_os = "linux")]
    assert_eq!(instructions.for_platform().len(), 1);
}

#[test]
fn test_dependency_check_creation() {
    let check = DependencyCheck {
        dependency: Dependency::NodeJs { min_version: None },
        status: DependencyStatus::Installed {
            version: Some("18.0.0".to_string()),
        },
        install_instructions: None,
    };

    assert_eq!(check.dependency.name(), "Node.js");
    match check.status {
        DependencyStatus::Installed { version } => {
            assert_eq!(version, Some("18.0.0".to_string()));
        }
        _ => panic!("Expected installed status"),
    }
}

#[test]
fn test_install_method_creation() {
    let method = InstallMethod {
        name: "homebrew".to_string(),
        command: "brew install node".to_string(),
        description: Some("Package manager for macOS".to_string()),
    };

    assert_eq!(method.name, "homebrew");
    assert_eq!(method.command, "brew install node");
    assert_eq!(
        method.description,
        Some("Package manager for macOS".to_string())
    );
}

#[test]
fn test_get_install_instructions_nodejs() {
    let dep = Dependency::NodeJs { min_version: None };
    let instructions = get_install_instructions(&dep);

    assert!(!instructions.windows.is_empty());
    assert!(!instructions.macos.is_empty());
    assert!(!instructions.linux.is_empty());

    // Check that each platform has appropriate methods
    assert!(instructions
        .windows
        .iter()
        .any(|m| m.name == "winget" || m.name == "chocolatey" || m.name == "download"));
    assert!(instructions
        .macos
        .iter()
        .any(|m| m.name == "homebrew" || m.name == "macports" || m.name == "download"));
    assert!(instructions
        .linux
        .iter()
        .any(|m| m.name == "apt" || m.name == "dnf" || m.name == "snap" || m.name == "nvm"));
}

#[test]
fn test_get_install_instructions_python() {
    let dep = Dependency::Python { min_version: None };
    let instructions = get_install_instructions(&dep);

    assert!(!instructions.windows.is_empty());
    assert!(!instructions.macos.is_empty());
    assert!(!instructions.linux.is_empty());

    // Check Python-specific methods
    assert!(instructions.windows.iter().any(|m| m.name == "winget"));
    assert!(instructions.macos.iter().any(|m| m.name == "pyenv"));
    assert!(instructions
        .linux
        .iter()
        .any(|m| m.command.contains("python3")));
}

#[test]
fn test_get_install_instructions_docker() {
    let dep = Dependency::Docker;
    let instructions = get_install_instructions(&dep);

    assert!(!instructions.windows.is_empty());
    assert!(!instructions.macos.is_empty());
    assert!(!instructions.linux.is_empty());

    // Check Docker-specific methods
    assert!(instructions
        .windows
        .iter()
        .any(|m| m.name == "docker-desktop"));
    assert!(instructions
        .macos
        .iter()
        .any(|m| m.name == "docker-desktop"));
    assert!(instructions.linux.iter().any(|m| m.name == "docker-ce"));
}

#[test]
fn test_get_install_instructions_git() {
    let dep = Dependency::Git;
    let instructions = get_install_instructions(&dep);

    assert!(!instructions.windows.is_empty());
    assert!(!instructions.macos.is_empty());
    assert!(!instructions.linux.is_empty());

    // Check Git-specific methods
    assert!(instructions
        .windows
        .iter()
        .any(|m| m.command.contains("Git")));
    assert!(instructions.macos.iter().any(|m| m.name == "xcode"));
    assert!(instructions.linux.iter().any(|m| m.command.contains("git")));
}

#[test]
fn test_dependency_equality() {
    let node1 = Dependency::NodeJs { min_version: None };
    let node2 = Dependency::NodeJs { min_version: None };
    let node3 = Dependency::NodeJs {
        min_version: Some("18.0.0".to_string()),
    };

    assert_eq!(node1, node2);
    assert_ne!(node1, node3);
    assert_ne!(node1, Dependency::Python { min_version: None });
}

#[test]
fn test_dependency_status_equality() {
    let status1 = DependencyStatus::Installed { version: None };
    let status2 = DependencyStatus::Installed { version: None };
    let status3 = DependencyStatus::Installed {
        version: Some("1.0".to_string()),
    };

    assert_eq!(status1, status2);
    assert_ne!(status1, status3);
    assert_ne!(status1, DependencyStatus::Missing);
}

// Mock implementation for testing the trait
struct MockChecker {
    result: DependencyCheck,
}

impl DependencyChecker for MockChecker {
    fn check(&self) -> anyhow::Result<DependencyCheck> {
        Ok(self.result.clone())
    }
}

#[test]
fn test_mock_dependency_checker() {
    let checker = MockChecker {
        result: DependencyCheck {
            dependency: Dependency::NodeJs { min_version: None },
            status: DependencyStatus::Installed {
                version: Some("18.0.0".to_string()),
            },
            install_instructions: None,
        },
    };

    let result = checker.check().unwrap();
    assert_eq!(result.dependency.name(), "Node.js");
    match result.status {
        DependencyStatus::Installed { version } => {
            assert_eq!(version, Some("18.0.0".to_string()));
        }
        _ => panic!("Expected installed status"),
    }
}

#[test]
fn test_install_instructions_descriptions() {
    let dep = Dependency::NodeJs { min_version: None };
    let instructions = get_install_instructions(&dep);

    // Verify that important methods have descriptions
    let winget = instructions
        .windows
        .iter()
        .find(|m| m.name == "winget")
        .unwrap();
    assert!(winget.description.is_some());

    let homebrew = instructions
        .macos
        .iter()
        .find(|m| m.name == "homebrew")
        .unwrap();
    assert!(homebrew.description.is_some());

    let apt = instructions.linux.iter().find(|m| m.name == "apt").unwrap();
    assert!(apt.description.is_some());
}
