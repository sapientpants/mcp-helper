//! Property-based tests for installation planning logic
//!
//! This module contains property tests that verify installation planning functions
//! work correctly for all possible inputs.

#[cfg(test)]
mod tests {
    use crate::core::installation::*;
    use crate::deps::{Dependency, DependencyStatus};
    use crate::server::{ServerMetadata, ServerType};
    use proptest::prelude::*;
    use std::collections::HashMap;

    // Strategy for generating client names
    fn client_name() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("Claude Desktop".to_string()),
            Just("VS Code".to_string()),
            Just("Cursor".to_string()),
            Just("Windsurf".to_string()),
            Just("Claude Code".to_string()),
        ]
    }

    // Strategy for generating server names
    fn server_name() -> impl Strategy<Value = String> {
        prop_oneof![
            "@[a-z]+/[a-z][a-z0-9-]*",
            "[a-z][a-z0-9-]*",
            "https://github\\.com/[a-z]+/[a-z]+/releases/v[0-9]+\\.[0-9]+\\.[0-9]+/binary",
        ]
    }

    // Strategy for generating dependencies
    fn dependency() -> impl Strategy<Value = Dependency> {
        prop_oneof![
            Just(Dependency::NodeJs { min_version: None }),
            Just(Dependency::NodeJs {
                min_version: Some("16.0.0".to_string())
            }),
            Just(Dependency::Python { min_version: None }),
            Just(Dependency::Docker {
                min_version: None,
                requires_compose: false
            }),
            Just(Dependency::Docker {
                min_version: None,
                requires_compose: true
            }),
        ]
    }

    // Strategy for generating dependency status
    fn dependency_status() -> impl Strategy<Value = DependencyStatus> {
        prop_oneof![
            Just(DependencyStatus::Installed {
                version: Some("18.0.0".to_string())
            }),
            Just(DependencyStatus::Installed { version: None }),
            Just(DependencyStatus::Missing),
            Just(DependencyStatus::VersionMismatch {
                installed: "16.0.0".to_string(),
                required: "18.0.0".to_string(),
            }),
            Just(DependencyStatus::ConfigurationRequired {
                issue: "PATH not set".to_string(),
                solution: "Add to PATH".to_string(),
            }),
        ]
    }

    // Strategy for generating installation plans
    prop_compose! {
        fn installation_plan()(
            server_name in server_name(),
            clients in prop::collection::vec(client_name(), 0..5),
            deps in prop::collection::vec(
                (dependency(), dependency_status()),
                0..5
            ),
        ) -> InstallationPlan {
            let dependencies = plan_dependencies(deps, false);

            InstallationPlan {
                server_name,
                server_type: ServerType::Npm {
                    package: "test-package".to_string(),
                    version: None,
                },
                target_clients: clients,
                dependencies,
                configuration: HashMap::new(),
            }
        }
    }

    proptest! {
        #[test]
        fn test_select_target_clients_returns_all(
            clients in prop::collection::vec(client_name(), 0..10)
        ) {
            let selected = select_target_clients(&clients);
            prop_assert_eq!(selected, clients);
        }

        #[test]
        fn test_determine_dependency_action_consistency(
            status in dependency_status(),
            auto_install in prop::bool::ANY,
        ) {
            let action1 = determine_dependency_action(&status, auto_install);
            let action2 = determine_dependency_action(&status, auto_install);

            // Same inputs should produce same outputs
            prop_assert_eq!(action1, action2);
        }

        #[test]
        fn test_determine_dependency_action_installed_always_satisfied(
            version in prop::option::of("[0-9]+\\.[0-9]+\\.[0-9]+"),
            auto_install in prop::bool::ANY,
        ) {
            let status = DependencyStatus::Installed { version };
            let action = determine_dependency_action(&status, auto_install);

            prop_assert_eq!(action, DependencyAction::AlreadySatisfied);
        }

        #[test]
        fn test_determine_dependency_action_missing_respects_auto_install(
            auto_install in prop::bool::ANY,
        ) {
            let status = DependencyStatus::Missing;
            let action = determine_dependency_action(&status, auto_install);

            if auto_install {
                prop_assert_eq!(action, DependencyAction::RequiresInstallation);
            } else {
                prop_assert!(matches!(action, DependencyAction::RequiresUserAction(_)));
            }
        }

        #[test]
        fn test_plan_dependencies_preserves_all_dependencies(
            deps in prop::collection::vec(
                (dependency(), dependency_status()),
                0..10
            ),
            auto_install in prop::bool::ANY,
        ) {
            let plans = plan_dependencies(deps.clone(), auto_install);

            // All dependencies should be in the plan
            prop_assert_eq!(plans.len(), deps.len());

            // Each dependency should have an appropriate action
            for (i, plan) in plans.iter().enumerate() {
                prop_assert_eq!(&plan.dependency, &deps[i].0);
                prop_assert_eq!(&plan.status, &deps[i].1);
            }
        }

        #[test]
        fn test_validate_installation_plan_empty_clients(
            server_name in server_name(),
            deps in prop::collection::vec(
                (dependency(), dependency_status()),
                0..5
            ),
        ) {
            let plan = InstallationPlan {
                server_name,
                server_type: ServerType::Npm {
                    package: "test".to_string(),
                    version: None,
                },
                target_clients: vec![], // Empty clients
                dependencies: plan_dependencies(deps, true),
                configuration: HashMap::new(),
            };

            let result = validate_installation_plan(&plan);
            prop_assert!(result.is_err());
            prop_assert!(result.unwrap_err().contains("No target clients"));
        }

        #[test]
        fn test_validate_installation_plan_with_clients_succeeds(
            server_name in server_name(),
            clients in prop::collection::vec(client_name(), 1..5),
            num_deps in 0usize..5,
        ) {
            // Generate only satisfied dependencies
            let deps: Vec<_> = (0..num_deps)
                .map(|_| (
                    Dependency::NodeJs { min_version: None },
                    DependencyStatus::Installed { version: Some("18.0.0".to_string()) }
                ))
                .collect();

            let plan = InstallationPlan {
                server_name,
                server_type: ServerType::Npm {
                    package: "test".to_string(),
                    version: None,
                },
                target_clients: clients,
                dependencies: plan_dependencies(deps, true),
                configuration: HashMap::new(),
            };

            let result = validate_installation_plan(&plan);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_calculate_installation_complexity_increases_with_factors(
            num_clients in 0usize..10,
            num_deps_needing_install in 0usize..5,
            num_config_fields in 0usize..10,
        ) {
            let base_plan = InstallationPlan {
                server_name: "test".to_string(),
                server_type: ServerType::Npm {
                    package: "test".to_string(),
                    version: None,
                },
                target_clients: vec!["Client".to_string(); num_clients],
                dependencies: vec![],
                configuration: HashMap::new(),
            };

            let base_complexity = calculate_installation_complexity(&base_plan);

            // Add dependencies that need installation
            let mut plan_with_deps = base_plan.clone();
            plan_with_deps.dependencies = (0..num_deps_needing_install)
                .map(|_| DependencyPlan {
                    dependency: Dependency::NodeJs { min_version: None },
                    status: DependencyStatus::Missing,
                    action: DependencyAction::RequiresInstallation,
                })
                .collect();

            let deps_complexity = calculate_installation_complexity(&plan_with_deps);
            prop_assert!(deps_complexity >= base_complexity);

            // Add configuration
            let mut plan_with_config = plan_with_deps.clone();
            for i in 0..num_config_fields {
                plan_with_config.configuration.insert(
                    format!("field{i}"),
                    "value".to_string()
                );
            }

            let final_complexity = calculate_installation_complexity(&plan_with_config);
            prop_assert!(final_complexity >= deps_complexity);
        }

        #[test]
        fn test_plan_installation_consistency(
            server_name in server_name(),
            clients in prop::collection::vec(client_name(), 1..5),
            deps in prop::collection::vec(
                (dependency(), dependency_status()),
                0..5
            ),
            auto_install in prop::bool::ANY,
        ) {
            let metadata = ServerMetadata {
                name: server_name.clone(),
                description: None,
                server_type: ServerType::Npm {
                    package: server_name.clone(),
                    version: None,
                },
                required_config: vec![],
                optional_config: vec![],
            };

            let plan1 = plan_installation(
                &server_name,
                ServerType::Npm {
                    package: server_name.clone(),
                    version: None,
                },
                &metadata,
                &clients,
                deps.clone(),
                auto_install,
            );

            let plan2 = plan_installation(
                &server_name,
                ServerType::Npm {
                    package: server_name.clone(),
                    version: None,
                },
                &metadata,
                &clients,
                deps,
                auto_install,
            );

            // Same inputs should produce same outputs
            prop_assert_eq!(plan1.server_name, plan2.server_name);
            prop_assert_eq!(plan1.target_clients, plan2.target_clients);
            prop_assert_eq!(plan1.dependencies.len(), plan2.dependencies.len());
        }
    }
}
