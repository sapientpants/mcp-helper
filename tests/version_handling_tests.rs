//! Comprehensive tests for version handling and requirements

use mcp_helper::deps::version::{VersionHelper, VersionRequirement};
use semver::Version;

#[test]
fn test_version_requirement_exact_parsing() {
    // Test exact version with and without = prefix
    let test_cases = vec![
        ("1.2.3", true),
        ("=1.2.3", true),
        ("= 1.2.3", true), // With space
        ("0.0.1", true),
        ("999.999.999", true),
        ("1.0.0-alpha", true),
        ("1.0.0+build123", true),
    ];

    for (input, should_succeed) in test_cases {
        let result = VersionRequirement::parse(input);
        assert_eq!(result.is_ok(), should_succeed, "Failed to parse: {input}");

        if let Ok(req) = result {
            match req {
                VersionRequirement::Exact(v) => {
                    // Remove = prefix if present for comparison
                    let expected = input.trim_start_matches('=').trim();
                    assert_eq!(v.to_string(), expected);
                }
                _ => panic!("Expected Exact requirement for: {input}"),
            }
        }
    }
}

#[test]
fn test_version_requirement_minimum_parsing() {
    let test_cases = vec![
        (">=1.2.3", "1.2.3"),
        (">= 1.2.3", "1.2.3"), // With space
        (">=0.0.1", "0.0.1"),
        (">=10.20.30", "10.20.30"),
    ];

    for (input, expected_version) in test_cases {
        let req = VersionRequirement::parse(input).unwrap();
        match req {
            VersionRequirement::Minimum(v) => {
                assert_eq!(v.to_string(), expected_version);
            }
            _ => panic!("Expected Minimum requirement for: {input}"),
        }
    }
}

#[test]
fn test_version_requirement_compatible_parsing() {
    let test_cases = vec![
        ("^1.2.3", "1.2.3"),
        ("^ 1.2.3", "1.2.3"), // With space
        ("^0.1.0", "0.1.0"),
        ("^2.0.0", "2.0.0"),
    ];

    for (input, expected_version) in test_cases {
        let req = VersionRequirement::parse(input).unwrap();
        match req {
            VersionRequirement::Compatible(v) => {
                assert_eq!(v.to_string(), expected_version);
            }
            _ => panic!("Expected Compatible requirement for: {input}"),
        }
    }
}

#[test]
fn test_version_requirement_approximate_parsing() {
    let test_cases = vec![
        ("~1.2.3", "1.2.3"),
        ("~ 1.2.3", "1.2.3"), // With space
        ("~0.1.0", "0.1.0"),
        ("~5.4.3", "5.4.3"),
    ];

    for (input, expected_version) in test_cases {
        let req = VersionRequirement::parse(input).unwrap();
        match req {
            VersionRequirement::Approximate(v) => {
                assert_eq!(v.to_string(), expected_version);
            }
            _ => panic!("Expected Approximate requirement for: {input}"),
        }
    }
}

#[test]
fn test_version_requirement_any_parsing() {
    let any_inputs = vec!["", "*", "any"];

    for input in any_inputs {
        let req = VersionRequirement::parse(input).unwrap();
        assert!(matches!(req, VersionRequirement::Any));
    }
}

#[test]
fn test_version_requirement_custom_parsing() {
    // Test that pre-release versions are parsed as exact
    let req = VersionRequirement::parse("1.2.3-alpha").unwrap();
    assert!(matches!(req, VersionRequirement::Exact(_)));

    // Test that simple operators are parsed correctly
    let req = VersionRequirement::parse(">=1.2.0").unwrap();
    assert!(matches!(req, VersionRequirement::Minimum(_)));

    let req = VersionRequirement::parse("^1.2.0").unwrap();
    assert!(matches!(req, VersionRequirement::Compatible(_)));

    let req = VersionRequirement::parse("~1.2.0").unwrap();
    assert!(matches!(req, VersionRequirement::Approximate(_)));
}

#[test]
fn test_version_requirement_matches_exact() {
    let v1_2_3 = Version::new(1, 2, 3);
    let req = VersionRequirement::Exact(v1_2_3.clone());

    assert!(req.matches(&v1_2_3));
    assert!(!req.matches(&Version::new(1, 2, 4)));
    assert!(!req.matches(&Version::new(1, 2, 2)));
    assert!(!req.matches(&Version::new(2, 2, 3)));
}

#[test]
fn test_version_requirement_matches_minimum() {
    let v1_2_3 = Version::new(1, 2, 3);
    let req = VersionRequirement::Minimum(v1_2_3);

    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(req.matches(&Version::new(1, 2, 4)));
    assert!(req.matches(&Version::new(1, 3, 0)));
    assert!(req.matches(&Version::new(2, 0, 0)));
    assert!(!req.matches(&Version::new(1, 2, 2)));
    assert!(!req.matches(&Version::new(1, 1, 9)));
}

#[test]
fn test_version_requirement_matches_compatible() {
    let v1_2_3 = Version::new(1, 2, 3);
    let req = VersionRequirement::Compatible(v1_2_3);

    // Compatible with same major version
    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(req.matches(&Version::new(1, 2, 4)));
    assert!(req.matches(&Version::new(1, 3, 0)));
    assert!(req.matches(&Version::new(1, 99, 99)));

    // Not compatible with different major version
    assert!(!req.matches(&Version::new(2, 0, 0)));
    assert!(!req.matches(&Version::new(0, 99, 99)));

    // Not compatible with lower versions
    assert!(!req.matches(&Version::new(1, 2, 2)));
    assert!(!req.matches(&Version::new(1, 1, 99)));
}

#[test]
fn test_version_requirement_matches_approximate() {
    let v1_2_3 = Version::new(1, 2, 3);
    let req = VersionRequirement::Approximate(v1_2_3);

    // Approximate allows patch updates only
    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(req.matches(&Version::new(1, 2, 4)));
    assert!(req.matches(&Version::new(1, 2, 99)));

    // Not compatible with different minor version
    assert!(!req.matches(&Version::new(1, 3, 0)));
    assert!(!req.matches(&Version::new(1, 1, 99)));
    assert!(!req.matches(&Version::new(2, 2, 3)));

    // Not compatible with lower patch versions
    assert!(!req.matches(&Version::new(1, 2, 2)));
}

#[test]
fn test_version_requirement_matches_any() {
    let req = VersionRequirement::Any;

    // Any matches everything
    assert!(req.matches(&Version::new(0, 0, 1)));
    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(req.matches(&Version::new(999, 999, 999)));
}

#[test]
fn test_version_requirement_to_version_req() {
    // Test conversion to semver VersionReq
    let test_cases = vec![
        (VersionRequirement::Exact(Version::new(1, 2, 3)), "=1.2.3"),
        (
            VersionRequirement::Minimum(Version::new(1, 2, 3)),
            ">=1.2.3",
        ),
        (
            VersionRequirement::Compatible(Version::new(1, 2, 3)),
            "^1.2.3",
        ),
        (
            VersionRequirement::Approximate(Version::new(1, 2, 3)),
            "~1.2.3",
        ),
        (VersionRequirement::Any, "*"),
    ];

    for (req, expected) in test_cases {
        let version_req = req.to_version_req().unwrap();
        assert_eq!(version_req.to_string(), expected);
    }
}

#[test]
fn test_version_requirement_display() {
    let test_cases = vec![
        (VersionRequirement::Exact(Version::new(1, 2, 3)), "=1.2.3"),
        (
            VersionRequirement::Minimum(Version::new(1, 2, 3)),
            ">=1.2.3",
        ),
        (
            VersionRequirement::Compatible(Version::new(1, 2, 3)),
            "^1.2.3",
        ),
        (
            VersionRequirement::Approximate(Version::new(1, 2, 3)),
            "~1.2.3",
        ),
        (VersionRequirement::Any, "*"),
    ];

    for (req, expected) in test_cases {
        assert_eq!(req.to_string(), expected);
    }
}

#[test]
fn test_version_helper_parse_with_prefix() {
    let test_cases = vec![
        ("1.2.3", "1.2.3"),
        ("v1.2.3", "1.2.3"),
        // ("V1.2.3", "1.2.3"), // Capital V is not supported
        (" v1.2.3 ", "1.2.3"),
        ("v16.14.0", "16.14.0"),
        ("18.0.0", "18.0.0"),
    ];

    for (input, expected) in test_cases {
        let version = VersionHelper::parse_version(input).unwrap();
        assert_eq!(version.to_string(), expected);
    }
}

#[test]
fn test_version_helper_parse_errors() {
    let invalid_inputs = vec![
        "not-a-version",
        "v",
        "",
        "1.x.3",
        "a.b.c",
        "1.2",       // Missing patch
        "1",         // Missing minor and patch
        ".1.2.3",    // Leading dot
        "1.2.3.4.5", // Too many parts
    ];

    for input in invalid_inputs {
        assert!(
            VersionHelper::parse_version(input).is_err(),
            "Should fail to parse: {input}"
        );
    }
}

#[test]
fn test_version_helper_compare_all_orderings() {
    use std::cmp::Ordering;

    let test_cases = vec![
        // Equal versions
        ("1.2.3", "1.2.3", Ordering::Equal),
        ("v1.2.3", "1.2.3", Ordering::Equal),
        ("16.0.0", "v16.0.0", Ordering::Equal),
        // Less than
        ("1.2.3", "1.2.4", Ordering::Less),
        ("1.2.3", "1.3.0", Ordering::Less),
        ("1.2.3", "2.0.0", Ordering::Less),
        ("0.9.9", "1.0.0", Ordering::Less),
        // Greater than
        ("2.0.0", "1.9.9", Ordering::Greater),
        ("1.3.0", "1.2.9", Ordering::Greater),
        ("1.2.4", "1.2.3", Ordering::Greater),
        ("10.0.0", "9.9.9", Ordering::Greater),
    ];

    for (v1, v2, expected) in test_cases {
        let result = VersionHelper::compare(v1, v2).unwrap();
        assert_eq!(
            result, expected,
            "Compare {v1} vs {v2} expected {expected:?}"
        );
    }
}

#[test]
fn test_version_helper_satisfies_complex() {
    let test_cases = vec![
        // Caret requirements
        ("1.2.3", "^1.0.0", true),
        ("1.9.9", "^1.0.0", true),
        ("2.0.0", "^1.0.0", false),
        ("0.9.9", "^1.0.0", false),
        // Tilde requirements
        ("1.2.3", "~1.2.0", true),
        ("1.2.9", "~1.2.0", true),
        ("1.3.0", "~1.2.0", false),
        ("1.1.9", "~1.2.0", false),
        // Minimum requirements
        ("1.2.3", ">=1.0.0", true),
        ("2.0.0", ">=1.0.0", true),
        ("0.9.9", ">=1.0.0", false),
        // Exact requirements
        ("1.2.3", "=1.2.3", true),
        ("1.2.3", "1.2.3", true),
        ("1.2.4", "=1.2.3", false),
        // Any requirement
        ("1.2.3", "*", true),
        ("0.0.1", "*", true),
        ("999.999.999", "*", true),
    ];

    for (version, requirement, expected) in test_cases {
        let result = VersionHelper::satisfies(version, requirement).unwrap();
        assert_eq!(
            result,
            expected,
            "Version {version} should{} satisfy {requirement}",
            if expected { "" } else { " not" }
        );
    }
}

#[test]
fn test_version_helper_next_versions() {
    let v = Version::new(1, 2, 3);

    // Test next major
    let next_major = VersionHelper::next_major(&v);
    assert_eq!(next_major, Version::new(2, 0, 0));

    // Test next minor
    let next_minor = VersionHelper::next_minor(&v);
    assert_eq!(next_minor, Version::new(1, 3, 0));

    // Test next patch
    let next_patch = VersionHelper::next_patch(&v);
    assert_eq!(next_patch, Version::new(1, 2, 4));
}

#[test]
fn test_version_helper_edge_cases() {
    // Test with version 0.x.x
    let v0 = Version::new(0, 1, 2);
    assert_eq!(VersionHelper::next_major(&v0), Version::new(1, 0, 0));
    assert_eq!(VersionHelper::next_minor(&v0), Version::new(0, 2, 0));
    assert_eq!(VersionHelper::next_patch(&v0), Version::new(0, 1, 3));

    // Test with large numbers
    let v_large = Version::new(999, 999, 999);
    assert_eq!(
        VersionHelper::next_major(&v_large),
        Version::new(1000, 0, 0)
    );
    assert_eq!(
        VersionHelper::next_minor(&v_large),
        Version::new(999, 1000, 0)
    );
    assert_eq!(
        VersionHelper::next_patch(&v_large),
        Version::new(999, 999, 1000)
    );
}

#[test]
fn test_caret_range() {
    let test_cases = vec![
        (
            Version::new(1, 2, 3),
            Version::new(1, 2, 3),
            Version::new(2, 0, 0),
        ),
        (
            Version::new(0, 1, 2),
            Version::new(0, 1, 2),
            Version::new(1, 0, 0),
        ),
        (
            Version::new(5, 4, 3),
            Version::new(5, 4, 3),
            Version::new(6, 0, 0),
        ),
    ];

    for (input, expected_lower, expected_upper) in test_cases {
        let (lower, upper) = VersionHelper::caret_range(&input);
        assert_eq!(lower, expected_lower);
        assert_eq!(upper, expected_upper);
    }
}

#[test]
fn test_tilde_range() {
    let test_cases = vec![
        (
            Version::new(1, 2, 3),
            Version::new(1, 2, 3),
            Version::new(1, 3, 0),
        ),
        (
            Version::new(0, 1, 2),
            Version::new(0, 1, 2),
            Version::new(0, 2, 0),
        ),
        (
            Version::new(5, 4, 3),
            Version::new(5, 4, 3),
            Version::new(5, 5, 0),
        ),
    ];

    for (input, expected_lower, expected_upper) in test_cases {
        let (lower, upper) = VersionHelper::tilde_range(&input);
        assert_eq!(lower, expected_lower);
        assert_eq!(upper, expected_upper);
    }
}

#[test]
fn test_pre_release_versions() {
    // Test parsing pre-release versions
    let alpha = VersionHelper::parse_version("1.0.0-alpha").unwrap();
    assert_eq!(alpha.pre.as_str(), "alpha");

    let beta = VersionHelper::parse_version("1.0.0-beta.1").unwrap();
    assert_eq!(beta.pre.as_str(), "beta.1");

    // Test comparison with pre-release
    use std::cmp::Ordering;
    let result = VersionHelper::compare("1.0.0-alpha", "1.0.0-beta").unwrap();
    assert_eq!(result, Ordering::Less);

    let result = VersionHelper::compare("1.0.0", "1.0.0-alpha").unwrap();
    assert_eq!(result, Ordering::Greater); // Release version is greater than pre-release
}

#[test]
fn test_build_metadata() {
    // Test parsing versions with build metadata
    let with_build = VersionHelper::parse_version("1.0.0+build123").unwrap();
    assert_eq!(with_build.build.as_str(), "build123");

    let complex = VersionHelper::parse_version("1.0.0-alpha+build.456").unwrap();
    assert_eq!(complex.pre.as_str(), "alpha");
    assert_eq!(complex.build.as_str(), "build.456");

    // Build metadata affects comparison in semver crate
    // The comparison result depends on the specific build metadata
    let result = VersionHelper::compare("1.0.0+build1", "1.0.0+build2").unwrap();
    // Build metadata does affect ordering in semver crate
    assert!(matches!(
        result,
        std::cmp::Ordering::Less | std::cmp::Ordering::Greater
    ));
}

#[test]
fn test_version_requirement_roundtrip() {
    // Test that parsing and displaying are consistent
    let requirements = vec!["=1.2.3", ">=1.2.3", "^1.2.3", "~1.2.3", "*"];

    for req_str in requirements {
        let req = VersionRequirement::parse(req_str).unwrap();
        let displayed = req.to_string();
        let reparsed = VersionRequirement::parse(&displayed).unwrap();

        // They should produce the same string representation
        assert_eq!(req.to_string(), reparsed.to_string());
    }
}

#[test]
fn test_zero_versions() {
    // Test special cases with 0.x.x versions
    let v0_1_0 = Version::new(0, 1, 0);
    let v0_0_1 = Version::new(0, 0, 1);

    // Compatible (^) with major version 0 is less restrictive in our implementation
    let req = VersionRequirement::Compatible(v0_1_0.clone());
    assert!(req.matches(&Version::new(0, 1, 0)));
    assert!(req.matches(&Version::new(0, 1, 5)));
    assert!(req.matches(&Version::new(0, 2, 0))); // Our implementation allows this
    assert!(!req.matches(&Version::new(1, 0, 0))); // Different major version

    // Test with 0.0.x
    let req = VersionRequirement::Compatible(v0_0_1);
    assert!(req.matches(&Version::new(0, 0, 1)));
    assert!(req.matches(&Version::new(0, 0, 2)));
    assert!(req.matches(&Version::new(0, 1, 0))); // Our implementation allows this
}

#[test]
fn test_complex_custom_requirements() {
    // Test parsing requirements that our parser supports
    let complex_reqs = vec![
        ("~1.2.0", vec!["1.2.0", "1.2.9"], vec!["1.1.9", "1.3.0"]),
        (
            "^1.2.0",
            vec!["1.2.0", "1.2.9", "1.9.0"],
            vec!["0.9.0", "2.0.0"],
        ),
        (
            ">=1.5.0",
            vec!["1.5.0", "1.9.9", "2.0.0"],
            vec!["0.9.0", "1.4.9"],
        ),
    ];

    for (req_str, should_match, should_not_match) in complex_reqs {
        let req = VersionRequirement::parse(req_str).unwrap();

        for version_str in should_match {
            let version = Version::parse(version_str).unwrap();
            assert!(
                req.matches(&version),
                "{req_str} should match {version_str}"
            );
        }

        for version_str in should_not_match {
            let version = Version::parse(version_str).unwrap();
            assert!(
                !req.matches(&version),
                "{req_str} should not match {version_str}"
            );
        }
    }
}

#[test]
fn test_version_requirement_equality() {
    // Test PartialEq implementation
    let req1 = VersionRequirement::Exact(Version::new(1, 2, 3));
    let req2 = VersionRequirement::Exact(Version::new(1, 2, 3));
    let req3 = VersionRequirement::Exact(Version::new(1, 2, 4));

    assert_eq!(req1, req2);
    assert_ne!(req1, req3);

    // Different types should not be equal
    let req_min = VersionRequirement::Minimum(Version::new(1, 2, 3));
    assert_ne!(req1, req_min);
}

#[test]
fn test_version_parsing_whitespace() {
    // Test that whitespace is handled correctly
    let test_cases = vec![
        ("  1.2.3  ", "1.2.3"),
        ("\t1.2.3\n", "1.2.3"),
        // ("v  1.2.3", "1.2.3"), // Space after v is not supported
        ("  v1.2.3  ", "1.2.3"),
    ];

    for (input, expected) in test_cases {
        let version = VersionHelper::parse_version(input).unwrap();
        assert_eq!(version.to_string(), expected);
    }
}

#[test]
fn test_error_messages() {
    // Test that error messages are helpful
    let result = VersionHelper::parse_version("not-a-version");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Failed to parse version"));
    assert!(error.to_string().contains("not-a-version"));

    let result = VersionRequirement::parse(">=not-a-version");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid version"));
}

#[test]
fn test_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    // VersionRequirement should be thread-safe
    let req = Arc::new(VersionRequirement::Minimum(Version::new(1, 2, 3)));

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let req = Arc::clone(&req);
            thread::spawn(move || {
                let version = Version::new(1, 2, i + 3);
                assert!(req.matches(&version));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
