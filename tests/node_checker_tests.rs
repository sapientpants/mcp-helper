use mcp_helper::deps::{Dependency, DependencyChecker, DependencyStatus, NodeChecker};

#[test]
fn test_node_checker_creation() {
    let checker = NodeChecker::new();
    // Can't easily test the actual check without Node.js installed
    // But we can verify the checker is created
    let _ = checker;
}

#[test]
fn test_node_checker_with_version() {
    let checker = NodeChecker::new().with_min_version("18.0.0".to_string());
    // Verify it's created with the version requirement
    let _ = checker;
}

#[test]
fn test_node_checker_implements_trait() {
    let checker = NodeChecker::new();
    // This will fail if Node.js is not installed, which is expected
    let _result = checker.check();
}

#[test]
fn test_dependency_status_scenarios() {
    // Test the different status types work correctly
    let installed = DependencyStatus::Installed {
        version: Some("18.0.0".to_string()),
    };
    assert!(matches!(installed, DependencyStatus::Installed { .. }));

    let missing = DependencyStatus::Missing;
    assert!(matches!(missing, DependencyStatus::Missing));

    let mismatch = DependencyStatus::VersionMismatch {
        installed: "16.0.0".to_string(),
        required: "18.0.0".to_string(),
    };
    assert!(matches!(mismatch, DependencyStatus::VersionMismatch { .. }));
}

#[test]
fn test_node_dependency_creation() {
    let dep1 = Dependency::NodeJs { min_version: None };
    assert_eq!(dep1.name(), "Node.js");

    let dep2 = Dependency::NodeJs {
        min_version: Some("16.0.0".to_string()),
    };
    assert_eq!(dep2.name(), "Node.js");
}
