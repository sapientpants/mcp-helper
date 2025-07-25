use mcp_helper::deps::{VersionHelper, VersionRequirement};
use semver::Version;

#[test]
fn test_version_requirement_parse_exact() {
    let req = VersionRequirement::parse("1.2.3").unwrap();
    match req {
        VersionRequirement::Exact(v) => assert_eq!(v, Version::new(1, 2, 3)),
        _ => panic!("Expected Exact requirement"),
    }

    let req = VersionRequirement::parse("=1.2.3").unwrap();
    match req {
        VersionRequirement::Exact(v) => assert_eq!(v, Version::new(1, 2, 3)),
        _ => panic!("Expected Exact requirement"),
    }
}

#[test]
fn test_version_requirement_parse_minimum() {
    let req = VersionRequirement::parse(">=1.2.3").unwrap();
    match req {
        VersionRequirement::Minimum(v) => assert_eq!(v, Version::new(1, 2, 3)),
        _ => panic!("Expected Minimum requirement"),
    }
}

#[test]
fn test_version_requirement_parse_compatible() {
    let req = VersionRequirement::parse("^1.2.3").unwrap();
    match req {
        VersionRequirement::Compatible(v) => assert_eq!(v, Version::new(1, 2, 3)),
        _ => panic!("Expected Compatible requirement"),
    }
}

#[test]
fn test_version_requirement_parse_approximate() {
    let req = VersionRequirement::parse("~1.2.3").unwrap();
    match req {
        VersionRequirement::Approximate(v) => assert_eq!(v, Version::new(1, 2, 3)),
        _ => panic!("Expected Approximate requirement"),
    }
}

#[test]
fn test_version_requirement_parse_any() {
    let req = VersionRequirement::parse("*").unwrap();
    assert!(matches!(req, VersionRequirement::Any));

    let req = VersionRequirement::parse("").unwrap();
    assert!(matches!(req, VersionRequirement::Any));

    let req = VersionRequirement::parse("any").unwrap();
    assert!(matches!(req, VersionRequirement::Any));
}

#[test]
fn test_version_requirement_matches_exact() {
    let req = VersionRequirement::Exact(Version::new(1, 2, 3));
    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(!req.matches(&Version::new(1, 2, 4)));
    assert!(!req.matches(&Version::new(1, 3, 0)));
}

#[test]
fn test_version_requirement_matches_minimum() {
    let req = VersionRequirement::Minimum(Version::new(1, 2, 3));
    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(req.matches(&Version::new(1, 2, 4)));
    assert!(req.matches(&Version::new(1, 3, 0)));
    assert!(req.matches(&Version::new(2, 0, 0)));
    assert!(!req.matches(&Version::new(1, 2, 2)));
}

#[test]
fn test_version_requirement_matches_compatible() {
    let req = VersionRequirement::Compatible(Version::new(1, 2, 3));
    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(req.matches(&Version::new(1, 2, 4)));
    assert!(req.matches(&Version::new(1, 3, 0)));
    assert!(req.matches(&Version::new(1, 99, 99)));
    assert!(!req.matches(&Version::new(2, 0, 0)));
    assert!(!req.matches(&Version::new(0, 9, 9)));
}

#[test]
fn test_version_requirement_matches_approximate() {
    let req = VersionRequirement::Approximate(Version::new(1, 2, 3));
    assert!(req.matches(&Version::new(1, 2, 3)));
    assert!(req.matches(&Version::new(1, 2, 4)));
    assert!(req.matches(&Version::new(1, 2, 99)));
    assert!(!req.matches(&Version::new(1, 3, 0)));
    assert!(!req.matches(&Version::new(2, 0, 0)));
}

#[test]
fn test_version_helper_parse_with_prefix() {
    assert_eq!(
        VersionHelper::parse_version("v1.2.3").unwrap(),
        Version::new(1, 2, 3)
    );
    assert_eq!(
        VersionHelper::parse_version(" v1.2.3 ").unwrap(),
        Version::new(1, 2, 3)
    );
    assert_eq!(
        VersionHelper::parse_version("1.2.3").unwrap(),
        Version::new(1, 2, 3)
    );
}

#[test]
fn test_version_helper_satisfies_complex() {
    // Compatible version tests
    assert!(VersionHelper::satisfies("1.2.3", "^1.0.0").unwrap());
    assert!(VersionHelper::satisfies("1.99.0", "^1.0.0").unwrap());
    assert!(!VersionHelper::satisfies("2.0.0", "^1.0.0").unwrap());

    // Approximate version tests
    assert!(VersionHelper::satisfies("1.2.3", "~1.2.0").unwrap());
    assert!(VersionHelper::satisfies("1.2.99", "~1.2.0").unwrap());
    assert!(!VersionHelper::satisfies("1.3.0", "~1.2.0").unwrap());

    // Minimum version tests
    assert!(VersionHelper::satisfies("2.0.0", ">=1.0.0").unwrap());
    assert!(VersionHelper::satisfies("1.0.0", ">=1.0.0").unwrap());
    assert!(!VersionHelper::satisfies("0.9.9", ">=1.0.0").unwrap());

    // Any version
    assert!(VersionHelper::satisfies("0.0.1", "*").unwrap());
    assert!(VersionHelper::satisfies("999.999.999", "*").unwrap());
}

#[test]
fn test_version_ranges() {
    let v = Version::new(1, 2, 3);

    // Caret range: ^1.2.3 means >=1.2.3 and <2.0.0
    let (lower, upper) = VersionHelper::caret_range(&v);
    assert_eq!(lower, Version::new(1, 2, 3));
    assert_eq!(upper, Version::new(2, 0, 0));

    // Tilde range: ~1.2.3 means >=1.2.3 and <1.3.0
    let (lower, upper) = VersionHelper::tilde_range(&v);
    assert_eq!(lower, Version::new(1, 2, 3));
    assert_eq!(upper, Version::new(1, 3, 0));
}

#[test]
fn test_version_requirement_display() {
    assert_eq!(
        VersionRequirement::Exact(Version::new(1, 2, 3)).to_string(),
        "=1.2.3"
    );
    assert_eq!(
        VersionRequirement::Minimum(Version::new(1, 2, 3)).to_string(),
        ">=1.2.3"
    );
    assert_eq!(
        VersionRequirement::Compatible(Version::new(1, 2, 3)).to_string(),
        "^1.2.3"
    );
    assert_eq!(
        VersionRequirement::Approximate(Version::new(1, 2, 3)).to_string(),
        "~1.2.3"
    );
    assert_eq!(VersionRequirement::Any.to_string(), "*");
}

#[test]
fn test_version_requirement_to_version_req() {
    let req = VersionRequirement::Compatible(Version::new(1, 2, 3));
    let version_req = req.to_version_req().unwrap();
    assert!(version_req.matches(&Version::new(1, 2, 3)));
    assert!(version_req.matches(&Version::new(1, 9, 9)));
    assert!(!version_req.matches(&Version::new(2, 0, 0)));
}

#[test]
fn test_complex_semver_requirements() {
    let req = VersionRequirement::parse(">1.0.0, <2.0.0").unwrap();
    match req {
        VersionRequirement::Custom(vr) => {
            assert!(vr.matches(&Version::new(1, 5, 0)));
            assert!(!vr.matches(&Version::new(0, 9, 0)));
            assert!(!vr.matches(&Version::new(2, 0, 0)));
        }
        _ => panic!("Expected Custom requirement"),
    }
}

#[test]
fn test_prerelease_versions() {
    let version = VersionHelper::parse_version("1.0.0-alpha.1").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    assert_eq!(version.pre.as_str(), "alpha.1");
}

#[test]
fn test_edge_cases() {
    // Empty version requirement defaults to Any
    let req = VersionRequirement::parse("").unwrap();
    assert!(matches!(req, VersionRequirement::Any));

    // Version with build metadata
    let version = VersionHelper::parse_version("1.0.0+20130313144700").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.build.as_str(), "20130313144700");

    // Invalid version should error
    assert!(VersionHelper::parse_version("not.a.version").is_err());
    assert!(VersionRequirement::parse(">=not.a.version").is_err());
}
