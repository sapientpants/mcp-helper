use crate::deps::{Dependency, DependencyChecker, DependencyStatus};
use crate::server::{RegistryEntry, ServerType};
use std::collections::HashMap;

/// Server suggestion engine for finding alternatives
pub struct ServerSuggestions {
    registry: HashMap<String, RegistryEntry>,
    similarity_cache: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub server: RegistryEntry,
    pub reason: SuggestionReason,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub enum SuggestionReason {
    LowerRequirements { missing_deps: Vec<String> },
    SameFunctionality { category: String },
    PopularAlternative { popularity_score: f64 },
    PlatformCompatible { platform: String },
    SimilarName { similarity: f64 },
}

impl ServerSuggestions {
    pub fn new() -> Self {
        Self {
            registry: Self::create_mock_registry(),
            similarity_cache: HashMap::new(),
        }
    }

    /// Suggest alternative servers based on dependency issues
    pub fn suggest_alternatives(
        &mut self,
        target_server: &str,
        failed_dependency: Option<&Dependency>,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // Get the target server entry if it exists
        let target_entry = self.registry.get(target_server);

        // Find alternatives based on different criteria
        if let Some(failed_dep) = failed_dependency {
            suggestions.extend(self.find_lower_requirement_alternatives(target_server, failed_dep));
        }

        if let Some(entry) = target_entry {
            suggestions.extend(self.find_same_category_alternatives(entry));
            suggestions.extend(self.find_popular_alternatives(entry));
        }

        suggestions.extend(self.find_similar_name_alternatives(target_server));
        suggestions.extend(self.find_platform_compatible_alternatives());

        // Remove duplicates and sort by score
        self.deduplicate_and_score(&mut suggestions);
        suggestions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top 5 suggestions
        suggestions.into_iter().take(5).collect()
    }

    fn find_lower_requirement_alternatives(
        &self,
        _target_server: &str,
        failed_dependency: &Dependency,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        for entry in self.registry.values() {
            // Check if this server has lower requirements
            if self.has_lower_requirements(entry, failed_dependency) {
                let missing_deps = self.get_missing_deps_for_failed_dependency(failed_dependency);
                suggestions.push(Suggestion {
                    server: entry.clone(),
                    reason: SuggestionReason::LowerRequirements { missing_deps },
                    score: self.calculate_requirement_score(entry, failed_dependency),
                });
            }
        }

        suggestions
    }

    fn find_same_category_alternatives(&self, target_entry: &RegistryEntry) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        for entry in self.registry.values() {
            if entry.name != target_entry.name && entry.category == target_entry.category {
                suggestions.push(Suggestion {
                    server: entry.clone(),
                    reason: SuggestionReason::SameFunctionality {
                        category: entry.category.clone(),
                    },
                    score: entry.popularity_score * 0.8, // Category match bonus
                });
            }
        }

        suggestions
    }

    fn find_popular_alternatives(&self, target_entry: &RegistryEntry) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        for entry in self.registry.values() {
            if entry.name != target_entry.name
                && entry.popularity_score > target_entry.popularity_score
                && self.has_similar_tags(entry, target_entry)
            {
                suggestions.push(Suggestion {
                    server: entry.clone(),
                    reason: SuggestionReason::PopularAlternative {
                        popularity_score: entry.popularity_score,
                    },
                    score: entry.popularity_score,
                });
            }
        }

        suggestions
    }

    fn find_similar_name_alternatives(&mut self, target_server: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        if let Some(similar_names) = self.similarity_cache.get(target_server) {
            for name in similar_names {
                if let Some(entry) = self.registry.get(name) {
                    let similarity = self.calculate_name_similarity(target_server, name);
                    suggestions.push(Suggestion {
                        server: entry.clone(),
                        reason: SuggestionReason::SimilarName { similarity },
                        score: similarity * entry.popularity_score,
                    });
                }
            }
        } else {
            // Calculate similarities and cache them
            let mut similar_names = Vec::new();
            for entry in self.registry.values() {
                let similarity = self.calculate_name_similarity(target_server, &entry.package_name);
                if similarity > 0.3 {
                    // Only consider reasonably similar names
                    similar_names.push(entry.package_name.clone());
                    suggestions.push(Suggestion {
                        server: entry.clone(),
                        reason: SuggestionReason::SimilarName { similarity },
                        score: similarity * entry.popularity_score,
                    });
                }
            }
            self.similarity_cache
                .insert(target_server.to_string(), similar_names);
        }

        suggestions
    }

    fn find_platform_compatible_alternatives(&self) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let current_platform = std::env::consts::OS;

        for entry in self.registry.values() {
            // This is a simplified check - in reality, we'd check platform support metadata
            let platform_compatible = match entry.server_type {
                ServerType::Npm { .. } => true, // NPM works everywhere with Node.js
                ServerType::Binary { .. } => true, // Binaries can be platform-specific
                ServerType::Python { .. } => true, // Python works everywhere
                ServerType::Docker { .. } => true, // Docker works everywhere (if installed)
            };

            if platform_compatible {
                suggestions.push(Suggestion {
                    server: entry.clone(),
                    reason: SuggestionReason::PlatformCompatible {
                        platform: current_platform.to_string(),
                    },
                    score: entry.popularity_score * 0.6, // Platform compatibility bonus
                });
            }
        }

        suggestions
    }

    fn has_lower_requirements(
        &self,
        entry: &RegistryEntry,
        failed_dependency: &Dependency,
    ) -> bool {
        // Simplified logic - in reality, we'd check metadata for actual requirements
        match (&entry.server_type, failed_dependency) {
            (ServerType::Npm { .. }, Dependency::Docker { .. }) => true, // NPM doesn't need Docker
            (ServerType::Python { .. }, Dependency::NodeJs { .. }) => true, // Python doesn't need Node.js
            (ServerType::Docker { .. }, Dependency::NodeJs { .. }) => true, // Docker doesn't need Node.js
            (ServerType::Docker { .. }, Dependency::Python { .. }) => true, // Docker doesn't need Python
            _ => false,
        }
    }

    fn get_missing_deps_for_failed_dependency(
        &self,
        failed_dependency: &Dependency,
    ) -> Vec<String> {
        vec![failed_dependency.name().to_string()]
    }

    fn calculate_requirement_score(
        &self,
        entry: &RegistryEntry,
        _failed_dependency: &Dependency,
    ) -> f64 {
        // Higher score for verified servers with good popularity
        let mut score = entry.popularity_score;
        if entry.verified {
            score *= 1.2;
        }
        score
    }

    fn has_similar_tags(&self, entry1: &RegistryEntry, entry2: &RegistryEntry) -> bool {
        entry1.tags.iter().any(|tag| entry2.tags.contains(tag))
    }

    fn calculate_name_similarity(&self, name1: &str, name2: &str) -> f64 {
        // Simple similarity calculation based on common substrings
        let name1_lower = name1.to_lowercase();
        let name2_lower = name2.to_lowercase();

        if name1_lower == name2_lower {
            return 1.0;
        }

        // Check for common words/parts
        let name1_parts: Vec<&str> = name1_lower.split(&['-', '_', '/', '@']).collect();
        let name2_parts: Vec<&str> = name2_lower.split(&['-', '_', '/', '@']).collect();

        let common_parts = name1_parts
            .iter()
            .filter(|part| name2_parts.contains(part))
            .count();

        let total_parts = name1_parts.len().max(name2_parts.len());

        if total_parts == 0 {
            0.0
        } else {
            common_parts as f64 / total_parts as f64
        }
    }

    fn deduplicate_and_score(&self, suggestions: &mut Vec<Suggestion>) {
        // Remove duplicates by server name
        let mut seen = std::collections::HashSet::new();
        suggestions.retain(|suggestion| seen.insert(suggestion.server.name.clone()));

        // Adjust scores based on multiple criteria
        for suggestion in suggestions.iter_mut() {
            let mut score_multiplier = 1.0;

            // Boost verified servers
            if suggestion.server.verified {
                score_multiplier *= 1.1;
            }

            // Boost recently updated servers
            // (In a real implementation, we'd parse the date)
            if suggestion.server.last_updated.contains("2024") {
                score_multiplier *= 1.05;
            }

            suggestion.score *= score_multiplier;
        }
    }

    /// Check if a suggested server's dependencies are available
    pub fn check_suggestion_feasibility(&self, suggestion: &Suggestion) -> SuggestionFeasibility {
        let dependency_checker: Box<dyn DependencyChecker> = match &suggestion.server.server_type {
            ServerType::Npm { .. } => Box::new(crate::deps::NodeChecker::new()),
            ServerType::Python { .. } => Box::new(crate::deps::PythonChecker::new()),
            ServerType::Docker { .. } => Box::new(crate::deps::DockerChecker::new()),
            ServerType::Binary { .. } => {
                // Binary servers typically have no dependencies
                return SuggestionFeasibility::Ready;
            }
        };

        match dependency_checker.check() {
            Ok(check) => match check.status {
                DependencyStatus::Installed { .. } => SuggestionFeasibility::Ready,
                DependencyStatus::Missing => SuggestionFeasibility::RequiresInstallation {
                    missing_deps: vec![check.dependency.name().to_string()],
                },
                DependencyStatus::VersionMismatch { .. } => {
                    SuggestionFeasibility::RequiresUpgrade {
                        outdated_deps: vec![check.dependency.name().to_string()],
                    }
                }
                DependencyStatus::ConfigurationRequired { .. } => {
                    SuggestionFeasibility::RequiresConfiguration {
                        config_issue: "Dependency configuration required".to_string(),
                    }
                }
            },
            Err(_) => SuggestionFeasibility::Unknown,
        }
    }

    fn create_mock_registry() -> HashMap<String, RegistryEntry> {
        let mut registry = HashMap::new();

        registry.insert(
            "@modelcontextprotocol/server-filesystem".to_string(),
            RegistryEntry {
                name: "Filesystem Server".to_string(),
                description: "MCP server for filesystem operations".to_string(),
                package_name: "@modelcontextprotocol/server-filesystem".to_string(),
                server_type: ServerType::Npm {
                    package: "@modelcontextprotocol/server-filesystem".to_string(),
                    version: None,
                },
                category: "File Management".to_string(),
                tags: vec![
                    "filesystem".to_string(),
                    "files".to_string(),
                    "directory".to_string(),
                ],
                popularity_score: 9.5,
                last_updated: "2024-01-15".to_string(),
                verified: true,
            },
        );

        registry.insert(
            "@anthropic/mcp-server-git".to_string(),
            RegistryEntry {
                name: "Git Server".to_string(),
                description: "MCP server for Git operations".to_string(),
                package_name: "@anthropic/mcp-server-git".to_string(),
                server_type: ServerType::Npm {
                    package: "@anthropic/mcp-server-git".to_string(),
                    version: None,
                },
                category: "Version Control".to_string(),
                tags: vec![
                    "git".to_string(),
                    "version-control".to_string(),
                    "repository".to_string(),
                ],
                popularity_score: 8.2,
                last_updated: "2024-01-08".to_string(),
                verified: true,
            },
        );

        registry.insert(
            "mcp-file-browser".to_string(),
            RegistryEntry {
                name: "File Browser".to_string(),
                description: "Python-based file browsing server".to_string(),
                package_name: "mcp-file-browser".to_string(),
                server_type: ServerType::Python {
                    package: "mcp-file-browser".to_string(),
                    version: None,
                },
                category: "File Management".to_string(),
                tags: vec![
                    "filesystem".to_string(),
                    "browser".to_string(),
                    "python".to_string(),
                ],
                popularity_score: 7.8,
                last_updated: "2024-01-12".to_string(),
                verified: false,
            },
        );

        registry.insert(
            "docker:mcp/universal-server".to_string(),
            RegistryEntry {
                name: "Universal MCP Server".to_string(),
                description: "Dockerized universal MCP server with multiple capabilities"
                    .to_string(),
                package_name: "docker:mcp/universal-server".to_string(),
                server_type: ServerType::Docker {
                    image: "mcp/universal-server".to_string(),
                    tag: Some("latest".to_string()),
                },
                category: "Multi-Purpose".to_string(),
                tags: vec![
                    "docker".to_string(),
                    "universal".to_string(),
                    "multi-purpose".to_string(),
                ],
                popularity_score: 8.9,
                last_updated: "2024-01-20".to_string(),
                verified: true,
            },
        );

        registry
    }
}

#[derive(Debug, Clone)]
pub enum SuggestionFeasibility {
    Ready,
    RequiresInstallation { missing_deps: Vec<String> },
    RequiresUpgrade { outdated_deps: Vec<String> },
    RequiresConfiguration { config_issue: String },
    Unknown,
}

impl std::fmt::Display for SuggestionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionReason::LowerRequirements { missing_deps } => {
                write!(f, "Doesn't require: {}", missing_deps.join(", "))
            }
            SuggestionReason::SameFunctionality { category } => {
                write!(f, "Same category ({category})")
            }
            SuggestionReason::PopularAlternative { popularity_score } => {
                write!(f, "Popular alternative (score: {popularity_score:.1})")
            }
            SuggestionReason::PlatformCompatible { platform } => {
                write!(f, "Compatible with {platform}")
            }
            SuggestionReason::SimilarName { similarity } => {
                write!(f, "Similar name ({:.0}% match)", similarity * 100.0)
            }
        }
    }
}

impl std::fmt::Display for SuggestionFeasibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionFeasibility::Ready => write!(f, "Ready to install"),
            SuggestionFeasibility::RequiresInstallation { missing_deps } => {
                write!(f, "Requires installation of: {}", missing_deps.join(", "))
            }
            SuggestionFeasibility::RequiresUpgrade { outdated_deps } => {
                write!(f, "Requires upgrade of: {}", outdated_deps.join(", "))
            }
            SuggestionFeasibility::RequiresConfiguration { config_issue } => {
                write!(f, "Requires configuration: {config_issue}")
            }
            SuggestionFeasibility::Unknown => write!(f, "Status unknown"),
        }
    }
}

impl Default for ServerSuggestions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_suggestions_creation() {
        let suggestions = ServerSuggestions::new();
        assert!(!suggestions.registry.is_empty());
        assert!(suggestions.similarity_cache.is_empty());
    }

    #[test]
    fn test_suggest_alternatives_with_failed_dependency() {
        let mut suggestions = ServerSuggestions::new();
        let failed_dep = Dependency::Docker {
            min_version: None,
            requires_compose: false,
        };

        let alternatives = suggestions.suggest_alternatives("test-server", Some(&failed_dep));
        assert!(!alternatives.is_empty());

        // Should find servers that don't require Docker
        let has_npm_alternative = alternatives
            .iter()
            .any(|s| matches!(s.server.server_type, ServerType::Npm { .. }));
        assert!(has_npm_alternative);
    }

    #[test]
    fn test_suggest_alternatives_by_category() {
        let mut suggestions = ServerSuggestions::new();
        let alternatives =
            suggestions.suggest_alternatives("@modelcontextprotocol/server-filesystem", None);
        assert!(!alternatives.is_empty());

        // Should find other file management servers
        let has_file_mgmt_alternative = alternatives.iter().any(|s| {
            s.server.category == "File Management" && s.server.name != "Filesystem Server"
        });
        assert!(has_file_mgmt_alternative);
    }

    #[test]
    fn test_calculate_name_similarity() {
        let suggestions = ServerSuggestions::new();

        assert_eq!(suggestions.calculate_name_similarity("test", "test"), 1.0);
        assert_eq!(
            suggestions.calculate_name_similarity("test", "different"),
            0.0
        );

        let similarity =
            suggestions.calculate_name_similarity("mcp-filesystem", "mcp-file-browser");
        assert!(similarity > 0.0 && similarity < 1.0);
    }

    #[test]
    fn test_has_similar_tags() {
        let suggestions = ServerSuggestions::new();

        let entry1 = RegistryEntry {
            name: "Test1".to_string(),
            description: "Test".to_string(),
            package_name: "test1".to_string(),
            server_type: ServerType::Npm {
                package: "test1".to_string(),
                version: None,
            },
            category: "Test".to_string(),
            tags: vec!["filesystem".to_string(), "files".to_string()],
            popularity_score: 1.0,
            last_updated: "2024-01-01".to_string(),
            verified: false,
        };

        let entry2 = RegistryEntry {
            name: "Test2".to_string(),
            description: "Test".to_string(),
            package_name: "test2".to_string(),
            server_type: ServerType::Npm {
                package: "test2".to_string(),
                version: None,
            },
            category: "Test".to_string(),
            tags: vec!["filesystem".to_string(), "database".to_string()],
            popularity_score: 1.0,
            last_updated: "2024-01-01".to_string(),
            verified: false,
        };

        assert!(suggestions.has_similar_tags(&entry1, &entry2));
    }

    #[test]
    fn test_suggestion_feasibility_display() {
        let feasibility = SuggestionFeasibility::Ready;
        assert_eq!(feasibility.to_string(), "Ready to install");

        let feasibility = SuggestionFeasibility::RequiresInstallation {
            missing_deps: vec!["Node.js".to_string()],
        };
        assert!(feasibility.to_string().contains("Node.js"));
    }

    #[test]
    fn test_suggestion_reason_display() {
        let reason = SuggestionReason::LowerRequirements {
            missing_deps: vec!["Docker".to_string()],
        };
        assert!(reason.to_string().contains("Docker"));

        let reason = SuggestionReason::SameFunctionality {
            category: "File Management".to_string(),
        };
        assert!(reason.to_string().contains("File Management"));
    }

    #[test]
    fn test_deduplicate_suggestions() {
        let suggestions = ServerSuggestions::new();
        let entry = suggestions.registry.values().next().unwrap().clone();

        let mut suggestion_list = vec![
            Suggestion {
                server: entry.clone(),
                reason: SuggestionReason::SameFunctionality {
                    category: "Test".to_string(),
                },
                score: 5.0,
            },
            Suggestion {
                server: entry.clone(),
                reason: SuggestionReason::PopularAlternative {
                    popularity_score: 8.0,
                },
                score: 6.0,
            },
        ];

        suggestions.deduplicate_and_score(&mut suggestion_list);
        assert_eq!(suggestion_list.len(), 1); // Should deduplicate
    }
}
