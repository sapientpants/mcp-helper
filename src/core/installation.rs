//! Pure installation planning and decision logic
//!
//! This module contains business logic for installation planning and decision making
//! without performing actual I/O operations.

use crate::deps::{Dependency, DependencyStatus};
use crate::server::{ServerMetadata, ServerType};
use std::collections::HashMap;

/// Represents an installation plan for a server
#[derive(Debug, Clone, PartialEq)]
pub struct InstallationPlan {
    pub server_name: String,
    pub server_type: ServerType,
    pub target_clients: Vec<String>,
    pub dependencies: Vec<DependencyPlan>,
    pub configuration: HashMap<String, String>,
}

/// Represents a dependency that needs to be handled
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyPlan {
    pub dependency: Dependency,
    pub status: DependencyStatus,
    pub action: DependencyAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyAction {
    AlreadySatisfied,
    RequiresInstallation,
    RequiresUserAction(String),
    Skip,
}

/// Plans installation steps based on server metadata and environment
pub fn plan_installation(
    server_name: &str,
    server_type: ServerType,
    _metadata: &ServerMetadata,
    available_clients: &[String],
    dependency_statuses: Vec<(Dependency, DependencyStatus)>,
    auto_install_deps: bool,
) -> InstallationPlan {
    let target_clients = select_target_clients(available_clients);
    let dependencies = plan_dependencies(dependency_statuses, auto_install_deps);

    InstallationPlan {
        server_name: server_name.to_string(),
        server_type,
        target_clients,
        dependencies,
        configuration: HashMap::new(), // Will be filled during configuration phase
    }
}

/// Selects which clients to install to based on availability and user preferences
pub fn select_target_clients(available_clients: &[String]) -> Vec<String> {
    // For now, return all available clients
    // In the future, this could include user preference logic
    available_clients.to_vec()
}

/// Plans dependency handling based on current status and installation preferences
pub fn plan_dependencies(
    dependency_statuses: Vec<(Dependency, DependencyStatus)>,
    auto_install_deps: bool,
) -> Vec<DependencyPlan> {
    dependency_statuses
        .into_iter()
        .map(|(dependency, status)| {
            let action = determine_dependency_action(&status, auto_install_deps);
            DependencyPlan {
                dependency,
                status,
                action,
            }
        })
        .collect()
}

/// Determines what action to take for a dependency based on its status
pub fn determine_dependency_action(
    status: &DependencyStatus,
    auto_install_deps: bool,
) -> DependencyAction {
    match status {
        DependencyStatus::Installed { .. } => DependencyAction::AlreadySatisfied,
        DependencyStatus::Missing => {
            if auto_install_deps {
                DependencyAction::RequiresInstallation
            } else {
                DependencyAction::RequiresUserAction("Install missing dependency".to_string())
            }
        }
        DependencyStatus::VersionMismatch {
            installed,
            required,
        } => DependencyAction::RequiresUserAction(format!(
            "Update dependency from {installed} to {required}"
        )),
        DependencyStatus::ConfigurationRequired { issue, solution: _ } => {
            DependencyAction::RequiresUserAction(format!("Configuration required: {issue}"))
        }
    }
}

/// Validates that an installation plan is feasible
pub fn validate_installation_plan(plan: &InstallationPlan) -> Result<(), String> {
    if plan.target_clients.is_empty() {
        return Err("No target clients selected for installation".to_string());
    }

    // Check if any dependencies require user action
    let requires_user_action: Vec<_> = plan
        .dependencies
        .iter()
        .filter(|dep| matches!(dep.action, DependencyAction::RequiresUserAction(_)))
        .collect();

    if !requires_user_action.is_empty() {
        let actions: Vec<String> = requires_user_action
            .iter()
            .filter_map(|dep| {
                if let DependencyAction::RequiresUserAction(action) = &dep.action {
                    Some(action.clone())
                } else {
                    None
                }
            })
            .collect();

        return Err(format!(
            "Installation requires user actions: {}",
            actions.join(", ")
        ));
    }

    Ok(())
}

/// Calculates installation complexity score (for UX decisions)
pub fn calculate_installation_complexity(plan: &InstallationPlan) -> u32 {
    let mut complexity = 0;

    // Base complexity
    complexity += 1;

    // Multiple clients add complexity
    if plan.target_clients.len() > 1 {
        complexity += plan.target_clients.len() as u32;
    }

    // Dependencies that need installation add complexity
    complexity += plan
        .dependencies
        .iter()
        .filter(|dep| matches!(dep.action, DependencyAction::RequiresInstallation))
        .count() as u32
        * 2;

    // Configuration fields add complexity
    complexity += plan.configuration.len() as u32;

    complexity
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> ServerMetadata {
        ServerMetadata {
            name: "test-server".to_string(),
            description: Some("Test server".to_string()),
            server_type: ServerType::Npm {
                package: "test-server".to_string(),
                version: None,
            },
            required_config: vec![],
            optional_config: vec![],
        }
    }

    #[test]
    fn test_plan_installation() {
        let metadata = create_test_metadata();
        let available_clients = vec!["Claude Desktop".to_string(), "VS Code".to_string()];
        let dependency_statuses = vec![(
            Dependency::NodeJs { min_version: None },
            DependencyStatus::Installed {
                version: Some("18.0.0".to_string()),
            },
        )];

        let plan = plan_installation(
            "test-server",
            ServerType::Npm {
                package: "test-server".to_string(),
                version: None,
            },
            &metadata,
            &available_clients,
            dependency_statuses,
            false,
        );

        assert_eq!(plan.server_name, "test-server");
        assert_eq!(plan.target_clients, available_clients);
        assert_eq!(plan.dependencies.len(), 1);
        assert_eq!(
            plan.dependencies[0].action,
            DependencyAction::AlreadySatisfied
        );
    }

    #[test]
    fn test_select_target_clients() {
        let available = vec!["Claude Desktop".to_string(), "VS Code".to_string()];
        let selected = select_target_clients(&available);
        assert_eq!(selected, available);
    }

    #[test]
    fn test_determine_dependency_action_installed() {
        let status = DependencyStatus::Installed {
            version: Some("18.0.0".to_string()),
        };
        let action = determine_dependency_action(&status, false);
        assert_eq!(action, DependencyAction::AlreadySatisfied);
    }

    #[test]
    fn test_determine_dependency_action_missing_auto_install() {
        let status = DependencyStatus::Missing;
        let action = determine_dependency_action(&status, true);
        assert_eq!(action, DependencyAction::RequiresInstallation);
    }

    #[test]
    fn test_determine_dependency_action_missing_manual() {
        let status = DependencyStatus::Missing;
        let action = determine_dependency_action(&status, false);
        assert!(matches!(action, DependencyAction::RequiresUserAction(_)));
    }

    #[test]
    fn test_validate_installation_plan_success() {
        let plan = InstallationPlan {
            server_name: "test".to_string(),
            server_type: ServerType::Npm {
                package: "test".to_string(),
                version: None,
            },
            target_clients: vec!["Claude Desktop".to_string()],
            dependencies: vec![DependencyPlan {
                dependency: Dependency::NodeJs { min_version: None },
                status: DependencyStatus::Installed {
                    version: Some("18.0.0".to_string()),
                },
                action: DependencyAction::AlreadySatisfied,
            }],
            configuration: HashMap::new(),
        };

        assert!(validate_installation_plan(&plan).is_ok());
    }

    #[test]
    fn test_validate_installation_plan_no_clients() {
        let plan = InstallationPlan {
            server_name: "test".to_string(),
            server_type: ServerType::Npm {
                package: "test".to_string(),
                version: None,
            },
            target_clients: vec![],
            dependencies: vec![],
            configuration: HashMap::new(),
        };

        let result = validate_installation_plan(&plan);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No target clients"));
    }

    #[test]
    fn test_calculate_installation_complexity() {
        let plan = InstallationPlan {
            server_name: "test".to_string(),
            server_type: ServerType::Npm {
                package: "test".to_string(),
                version: None,
            },
            target_clients: vec!["Claude Desktop".to_string(), "VS Code".to_string()],
            dependencies: vec![DependencyPlan {
                dependency: Dependency::NodeJs { min_version: None },
                status: DependencyStatus::Missing,
                action: DependencyAction::RequiresInstallation,
            }],
            configuration: {
                let mut config = HashMap::new();
                config.insert("api_key".to_string(), "test".to_string());
                config
            },
        };

        // Base (1) + Multiple clients (2) + Dependency installation (2) + Config (1) = 6
        assert_eq!(calculate_installation_complexity(&plan), 6);
    }
}
