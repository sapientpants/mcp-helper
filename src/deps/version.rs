use anyhow::{anyhow, Result};
use semver::{Version, VersionReq};
use std::fmt;

/// Represents a version requirement for a dependency
#[derive(Debug, Clone, PartialEq)]
pub enum VersionRequirement {
    /// Exact version match
    Exact(Version),
    /// Minimum version (>=)
    Minimum(Version),
    /// Compatible version (^) - allows patch and minor updates
    Compatible(Version),
    /// Approximately equivalent (~) - allows patch updates only
    Approximate(Version),
    /// Custom semver requirement
    Custom(VersionReq),
    /// Any version is acceptable
    Any,
}

impl VersionRequirement {
    /// Parse a version requirement string
    pub fn parse(input: &str) -> Result<Self> {
        if input.is_empty() || input == "*" || input == "any" {
            return Ok(VersionRequirement::Any);
        }

        // Check for operators
        if let Some(version_str) = input.strip_prefix(">=") {
            let version = Version::parse(version_str.trim())
                .map_err(|e| anyhow!("Invalid version '{}': {}", version_str, e))?;
            Ok(VersionRequirement::Minimum(version))
        } else if let Some(version_str) = input.strip_prefix('^') {
            let version = Version::parse(version_str.trim())
                .map_err(|e| anyhow!("Invalid version '{}': {}", version_str, e))?;
            Ok(VersionRequirement::Compatible(version))
        } else if let Some(version_str) = input.strip_prefix('~') {
            let version = Version::parse(version_str.trim())
                .map_err(|e| anyhow!("Invalid version '{}': {}", version_str, e))?;
            Ok(VersionRequirement::Approximate(version))
        } else if let Some(version_str) = input.strip_prefix('=') {
            let version = Version::parse(version_str.trim())
                .map_err(|e| anyhow!("Invalid version '{}': {}", version_str, e))?;
            Ok(VersionRequirement::Exact(version))
        } else {
            // Try to parse as exact version first
            if let Ok(version) = Version::parse(input) {
                Ok(VersionRequirement::Exact(version))
            } else {
                // Try as a semver requirement
                let req = VersionReq::parse(input)
                    .map_err(|e| anyhow!("Invalid version requirement '{}': {}", input, e))?;
                Ok(VersionRequirement::Custom(req))
            }
        }
    }

    /// Check if a version satisfies this requirement
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            VersionRequirement::Exact(v) => version == v,
            VersionRequirement::Minimum(v) => version >= v,
            VersionRequirement::Compatible(v) => {
                // Compatible: ^1.2.3 means >=1.2.3 and <2.0.0
                version.major == v.major && version >= v
            }
            VersionRequirement::Approximate(v) => {
                // Approximate: ~1.2.3 means >=1.2.3 and <1.3.0
                version.major == v.major && version.minor == v.minor && version >= v
            }
            VersionRequirement::Custom(req) => req.matches(version),
            VersionRequirement::Any => true,
        }
    }

    /// Convert to a semver VersionReq for compatibility
    pub fn to_version_req(&self) -> Result<VersionReq> {
        match self {
            VersionRequirement::Exact(v) => VersionReq::parse(&format!("={v}"))
                .map_err(|e| anyhow!("Failed to create version req: {}", e)),
            VersionRequirement::Minimum(v) => VersionReq::parse(&format!(">={v}"))
                .map_err(|e| anyhow!("Failed to create version req: {}", e)),
            VersionRequirement::Compatible(v) => VersionReq::parse(&format!("^{v}"))
                .map_err(|e| anyhow!("Failed to create version req: {}", e)),
            VersionRequirement::Approximate(v) => VersionReq::parse(&format!("~{v}"))
                .map_err(|e| anyhow!("Failed to create version req: {}", e)),
            VersionRequirement::Custom(req) => Ok(req.clone()),
            VersionRequirement::Any => {
                VersionReq::parse("*").map_err(|e| anyhow!("Failed to create version req: {}", e))
            }
        }
    }
}

impl fmt::Display for VersionRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionRequirement::Exact(v) => write!(f, "={v}"),
            VersionRequirement::Minimum(v) => write!(f, ">={v}"),
            VersionRequirement::Compatible(v) => write!(f, "^{v}"),
            VersionRequirement::Approximate(v) => write!(f, "~{v}"),
            VersionRequirement::Custom(req) => write!(f, "{req}"),
            VersionRequirement::Any => write!(f, "*"),
        }
    }
}

/// Helper functions for version operations
pub struct VersionHelper;

impl VersionHelper {
    /// Parse a version string, handling common prefixes like 'v'
    pub fn parse_version(version_str: &str) -> Result<Version> {
        let cleaned = version_str.trim().trim_start_matches('v');
        Version::parse(cleaned)
            .map_err(|e| anyhow!("Failed to parse version '{}': {}", version_str, e))
    }

    /// Compare two versions and return ordering
    pub fn compare(v1: &str, v2: &str) -> Result<std::cmp::Ordering> {
        let version1 = Self::parse_version(v1)?;
        let version2 = Self::parse_version(v2)?;
        Ok(version1.cmp(&version2))
    }

    /// Check if a version satisfies a requirement string
    pub fn satisfies(version: &str, requirement: &str) -> Result<bool> {
        let ver = Self::parse_version(version)?;
        let req = VersionRequirement::parse(requirement)?;
        Ok(req.matches(&ver))
    }

    /// Get the next major version
    pub fn next_major(version: &Version) -> Version {
        Version::new(version.major + 1, 0, 0)
    }

    /// Get the next minor version
    pub fn next_minor(version: &Version) -> Version {
        Version::new(version.major, version.minor + 1, 0)
    }

    /// Get the next patch version
    pub fn next_patch(version: &Version) -> Version {
        Version::new(version.major, version.minor, version.patch + 1)
    }

    /// Create a version range for caret (^) requirements
    pub fn caret_range(version: &Version) -> (Version, Version) {
        let lower = version.clone();
        let upper = Self::next_major(version);
        (lower, upper)
    }

    /// Create a version range for tilde (~) requirements
    pub fn tilde_range(version: &Version) -> (Version, Version) {
        let lower = version.clone();
        let upper = Self::next_minor(version);
        (lower, upper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_requirement_parse() {
        // Exact versions
        let req = VersionRequirement::parse("1.2.3").unwrap();
        assert!(matches!(req, VersionRequirement::Exact(_)));

        let req = VersionRequirement::parse("=1.2.3").unwrap();
        assert!(matches!(req, VersionRequirement::Exact(_)));

        // Minimum versions
        let req = VersionRequirement::parse(">=1.2.3").unwrap();
        assert!(matches!(req, VersionRequirement::Minimum(_)));

        // Compatible versions
        let req = VersionRequirement::parse("^1.2.3").unwrap();
        assert!(matches!(req, VersionRequirement::Compatible(_)));

        // Approximate versions
        let req = VersionRequirement::parse("~1.2.3").unwrap();
        assert!(matches!(req, VersionRequirement::Approximate(_)));

        // Any version
        let req = VersionRequirement::parse("*").unwrap();
        assert!(matches!(req, VersionRequirement::Any));

        let req = VersionRequirement::parse("").unwrap();
        assert!(matches!(req, VersionRequirement::Any));
    }

    #[test]
    fn test_version_requirement_matches() {
        let v1_2_3 = Version::new(1, 2, 3);
        let v1_2_4 = Version::new(1, 2, 4);
        let v1_3_0 = Version::new(1, 3, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        // Exact match
        let req = VersionRequirement::Exact(v1_2_3.clone());
        assert!(req.matches(&v1_2_3));
        assert!(!req.matches(&v1_2_4));

        // Minimum version
        let req = VersionRequirement::Minimum(v1_2_3.clone());
        assert!(req.matches(&v1_2_3));
        assert!(req.matches(&v1_2_4));
        assert!(req.matches(&v2_0_0));

        // Compatible version (^)
        let req = VersionRequirement::Compatible(v1_2_3.clone());
        assert!(req.matches(&v1_2_3));
        assert!(req.matches(&v1_2_4));
        assert!(req.matches(&v1_3_0));
        assert!(!req.matches(&v2_0_0));

        // Approximate version (~)
        let req = VersionRequirement::Approximate(v1_2_3.clone());
        assert!(req.matches(&v1_2_3));
        assert!(req.matches(&v1_2_4));
        assert!(!req.matches(&v1_3_0));
        assert!(!req.matches(&v2_0_0));
    }

    #[test]
    fn test_version_helper_parse() {
        assert_eq!(
            VersionHelper::parse_version("1.2.3").unwrap(),
            Version::new(1, 2, 3)
        );
        assert_eq!(
            VersionHelper::parse_version("v1.2.3").unwrap(),
            Version::new(1, 2, 3)
        );
        assert_eq!(
            VersionHelper::parse_version(" v1.2.3 ").unwrap(),
            Version::new(1, 2, 3)
        );
    }

    #[test]
    fn test_version_helper_compare() {
        use std::cmp::Ordering;

        assert_eq!(
            VersionHelper::compare("1.2.3", "1.2.3").unwrap(),
            Ordering::Equal
        );
        assert_eq!(
            VersionHelper::compare("1.2.3", "1.2.4").unwrap(),
            Ordering::Less
        );
        assert_eq!(
            VersionHelper::compare("2.0.0", "1.9.9").unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn test_version_helper_satisfies() {
        assert!(VersionHelper::satisfies("1.2.3", "^1.0.0").unwrap());
        assert!(VersionHelper::satisfies("1.2.3", "~1.2.0").unwrap());
        assert!(VersionHelper::satisfies("1.2.3", ">=1.0.0").unwrap());
        assert!(!VersionHelper::satisfies("1.2.3", "^2.0.0").unwrap());
        assert!(!VersionHelper::satisfies("1.3.0", "~1.2.0").unwrap());
    }

    #[test]
    fn test_version_helper_next() {
        let v = Version::new(1, 2, 3);

        assert_eq!(VersionHelper::next_major(&v), Version::new(2, 0, 0));
        assert_eq!(VersionHelper::next_minor(&v), Version::new(1, 3, 0));
        assert_eq!(VersionHelper::next_patch(&v), Version::new(1, 2, 4));
    }

    #[test]
    fn test_version_ranges() {
        let v = Version::new(1, 2, 3);

        let (lower, upper) = VersionHelper::caret_range(&v);
        assert_eq!(lower, Version::new(1, 2, 3));
        assert_eq!(upper, Version::new(2, 0, 0));

        let (lower, upper) = VersionHelper::tilde_range(&v);
        assert_eq!(lower, Version::new(1, 2, 3));
        assert_eq!(upper, Version::new(1, 3, 0));
    }

    #[test]
    fn test_complex_version_requirements() {
        // Test custom semver requirements
        let req = VersionRequirement::parse(">1.0.0, <2.0.0").unwrap();
        match req {
            VersionRequirement::Custom(vr) => {
                assert!(vr.matches(&Version::new(1, 5, 0)));
                assert!(!vr.matches(&Version::new(2, 0, 0)));
            }
            _ => panic!("Expected Custom requirement"),
        }
    }

    #[test]
    fn test_display() {
        let req = VersionRequirement::Exact(Version::new(1, 2, 3));
        assert_eq!(req.to_string(), "=1.2.3");

        let req = VersionRequirement::Minimum(Version::new(1, 2, 3));
        assert_eq!(req.to_string(), ">=1.2.3");

        let req = VersionRequirement::Compatible(Version::new(1, 2, 3));
        assert_eq!(req.to_string(), "^1.2.3");

        let req = VersionRequirement::Approximate(Version::new(1, 2, 3));
        assert_eq!(req.to_string(), "~1.2.3");

        let req = VersionRequirement::Any;
        assert_eq!(req.to_string(), "*");
    }
}
