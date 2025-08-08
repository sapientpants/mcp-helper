//! Property-based tests for server module functions
//!
//! This module contains property tests for version parsing and other server-related functions.

#[cfg(test)]
mod tests {
    use crate::server::{detect_server_type, parse_npm_package, ServerType};
    use proptest::prelude::*;

    // Strategy for generating NPM package names with versions
    prop_compose! {
        fn npm_package_with_version()(
            is_scoped in prop::bool::ANY,
            scope in "[a-z][a-z0-9-]*",
            name in "[a-z][a-z0-9-._]*",
            has_version in prop::bool::ANY,
            version in "[0-9]+\\.[0-9]+\\.[0-9]+(-[a-zA-Z0-9.-]+)?(\\+[a-zA-Z0-9.-]+)?",
        ) -> String {
            let package = if is_scoped {
                format!("@{scope}/{name}")
            } else {
                name
            };

            if has_version {
                format!("{package}@{version}")
            } else {
                package
            }
        }
    }

    // Strategy for generating docker image names
    prop_compose! {
        fn docker_image_spec()(
            registry in prop::option::of("[a-z0-9.-]+\\.[a-z]{2,}"),
            namespace in prop::option::of("[a-z0-9-]+"),
            repo in "[a-z][a-z0-9-]*",
            has_tag in prop::bool::ANY,
            tag in "[a-zA-Z0-9._-]+",
        ) -> String {
            let mut parts = vec![];
            if let Some(r) = registry {
                parts.push(r);
            }
            if let Some(n) = namespace {
                parts.push(n);
            }
            parts.push(repo);

            let base = parts.join("/");
            let full = if has_tag {
                format!("{base}:{tag}")
            } else {
                base
            };

            format!("docker:{full}")
        }
    }

    // Strategy for generating binary URLs
    fn binary_url() -> impl Strategy<Value = String> {
        "https://github\\.com/[a-z]+/[a-z]+/releases/download/v[0-9]+\\.[0-9]+\\.[0-9]+/[a-z-]+"
    }

    // Strategy for generating Python package names
    fn python_package() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_-]*\\.py"
    }

    proptest! {
        #[test]
        fn test_parse_npm_package_roundtrip(
            package in npm_package_with_version()
        ) {
            let (parsed_package, parsed_version) = parse_npm_package(&package);

            // Reconstruct the original
            let reconstructed = if let Some(version) = parsed_version {
                format!("{parsed_package}@{version}")
            } else {
                parsed_package.clone()
            };

            // For packages without version, should match exactly
            if !package.contains('@') || package.starts_with('@') && package.chars().filter(|&c| c == '@').count() == 1 {
                prop_assert_eq!(reconstructed, package);
            } else {
                // For packages with version, the reconstructed should match
                prop_assert_eq!(reconstructed, package);
            }
        }

        #[test]
        fn test_parse_npm_package_scoped_packages(
            scope in "[a-z][a-z0-9-]*",
            name in "[a-z][a-z0-9-._]*",
            version in prop::option::of("[0-9]+\\.[0-9]+\\.[0-9]+"),
        ) {
            let package = if let Some(ref v) = version {
                format!("@{scope}/{name}@{v}")
            } else {
                format!("@{scope}/{name}")
            };

            let (parsed_package, parsed_version) = parse_npm_package(&package);

            prop_assert_eq!(parsed_package, format!("@{}/{}", scope, name));
            prop_assert_eq!(parsed_version, version);
        }

        #[test]
        fn test_parse_npm_package_simple_packages(
            name in "[a-z][a-z0-9-._]*",
            version in prop::option::of("[0-9]+\\.[0-9]+\\.[0-9]+"),
        ) {
            let package = if let Some(ref v) = version {
                format!("{name}@{v}")
            } else {
                name.clone()
            };

            let (parsed_package, parsed_version) = parse_npm_package(&package);

            prop_assert_eq!(parsed_package, name);
            prop_assert_eq!(parsed_version, version);
        }

        #[test]
        fn test_detect_server_type_npm(
            package in npm_package_with_version()
        ) {
            // Skip if it looks like a URL or docker image
            prop_assume!(!package.starts_with("http"));
            prop_assume!(!package.starts_with("docker:"));
            prop_assume!(!package.ends_with(".py"));

            let server_type = detect_server_type(&package);

            match server_type {
                ServerType::Npm { .. } => {},
                _ => prop_assert!(false, "Expected NPM server type"),
            }

            if let ServerType::Npm { package: detected_package, version: detected_version } = server_type {
                let (expected_package, expected_version) = parse_npm_package(&package);
                prop_assert_eq!(detected_package, expected_package);
                prop_assert_eq!(detected_version, expected_version);
            }
        }

        #[test]
        fn test_detect_server_type_docker(
            image_spec in docker_image_spec()
        ) {
            let server_type = detect_server_type(&image_spec);

            match server_type {
                ServerType::Docker { .. } => {},
                _ => prop_assert!(false, "Expected Docker server type"),
            }

            if let ServerType::Docker { image, tag } = server_type {
                // Remove "docker:" prefix for validation
                let spec_without_prefix = image_spec.strip_prefix("docker:").unwrap();

                if spec_without_prefix.contains(':') {
                    let parts: Vec<&str> = spec_without_prefix.rsplitn(2, ':').collect();
                    prop_assert_eq!(image, parts[1]);
                    prop_assert_eq!(tag, Some(parts[0].to_string()));
                } else {
                    prop_assert_eq!(image, spec_without_prefix);
                    prop_assert_eq!(tag, Some("latest".to_string()));
                }
            }
        }

        #[test]
        fn test_detect_server_type_binary(
            url in binary_url()
        ) {
            let server_type = detect_server_type(&url);

            match server_type {
                ServerType::Binary { .. } => {},
                _ => prop_assert!(false, "Expected Binary server type"),
            }

            if let ServerType::Binary { url: detected_url, checksum } = server_type {
                prop_assert_eq!(detected_url, url);
                prop_assert_eq!(checksum, None);
            }
        }

        #[test]
        fn test_detect_server_type_python(
            package in python_package()
        ) {
            let server_type = detect_server_type(&package);

            match server_type {
                ServerType::Python { .. } => {},
                _ => prop_assert!(false, "Expected Python server type"),
            }

            if let ServerType::Python { package: detected_package, version } = server_type {
                prop_assert_eq!(detected_package, package);
                prop_assert_eq!(version, None);
            }
        }

        #[test]
        fn test_server_type_detection_consistency(
            input in prop::string::string_regex("[a-zA-Z0-9@/.:_-]+").unwrap()
        ) {
            let type1 = detect_server_type(&input);
            let type2 = detect_server_type(&input);

            // Detection should be deterministic
            prop_assert_eq!(type1, type2);
        }

        #[test]
        fn test_npm_version_parsing_edge_cases(
            package in "[a-z][a-z0-9-._]*",
            major in 0u32..100,
            minor in 0u32..100,
            patch in 0u32..100,
            prerelease in prop::option::of("[a-zA-Z0-9.-]+"),
            build in prop::option::of("[a-zA-Z0-9.-]+"),
        ) {
            let mut version = format!("{major}.{minor}.{patch}");
            if let Some(pre) = prerelease {
                version.push('-');
                version.push_str(&pre);
            }
            if let Some(b) = build {
                version.push('+');
                version.push_str(&b);
            }

            let full_package = format!("{package}@{version}");
            let (parsed_package, parsed_version) = parse_npm_package(&full_package);

            prop_assert_eq!(parsed_package, package);
            prop_assert_eq!(parsed_version, Some(version));
        }
    }
}
