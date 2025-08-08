//! Property-based tests for validation logic
//!
//! This module contains property tests that verify validation functions work correctly
//! for all possible inputs within defined constraints.

#[cfg(test)]
mod tests {
    use crate::core::validation::*;
    use crate::server::ServerType;
    use proptest::prelude::*;

    // Strategy for generating valid NPM package names
    prop_compose! {
        fn valid_npm_package_name()(
            is_scoped in prop::bool::ANY,
            scope in "[a-z][a-z0-9-]*",
            name in "[a-z][a-z0-9-._]*",
        ) -> String {
            if is_scoped {
                format!("@{scope}/{name}")
            } else {
                name
            }
        }
    }

    // Strategy for generating potentially invalid NPM package names
    prop_compose! {
        fn any_npm_package_name()(
            prefix in prop::option::of("[._@]"),
            body in "[a-zA-Z0-9-._/@]*",
        ) -> String {
            match prefix {
                Some(p) => format!("{p}{body}"),
                None => body,
            }
        }
    }

    // Strategy for generating Docker image names
    prop_compose! {
        fn docker_image_name()(
            registry in prop::option::of("[a-z0-9.-]+\\.[a-z]{2,}"),
            namespace in prop::option::of("[a-z0-9-]+"),
            repo in "[a-z][a-z0-9-]*",
            tag in prop::option::of(":[a-zA-Z0-9._-]+"),
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
            match tag {
                Some(t) => format!("{base}{t}"),
                None => base,
            }
        }
    }

    // Strategy for generating URLs
    prop_compose! {
        fn url_string()(
            protocol in prop::option::of("https?"),
            host in "[a-zA-Z0-9.-]+",
            port in prop::option::of(1000u16..9999),
            path in prop::option::of("/[a-zA-Z0-9/.?&=-]*"),
        ) -> String {
            let protocol = protocol.unwrap_or("https".to_string());
            let mut url = format!("{protocol}://{host}");
            if let Some(p) = port {
                url.push_str(&format!(":{p}"));
            }
            if let Some(path) = path {
                url.push_str(&path);
            }
            url
        }
    }

    proptest! {
        #[test]
        fn test_server_name_validation_length(
            name in prop::string::string_regex("[a-zA-Z0-9@/._: -]{1,256}").unwrap()
        ) {
            let result = validate_server_name(&name);
            if name.contains("..") {
                prop_assert!(result.is_err());
                prop_assert!(result.unwrap_err().contains("path traversal"));
            } else {
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn test_server_name_validation_too_long(
            name in prop::string::string_regex("[a-zA-Z0-9@/._: -]{257,500}").unwrap()
        ) {
            let result = validate_server_name(&name);
            prop_assert!(result.is_err());
            prop_assert!(result.unwrap_err().contains("too long"));
        }

        #[test]
        fn test_npm_package_name_validation_valid(
            name in valid_npm_package_name()
        ) {
            let result = validate_npm_package_name(&name);
            if name.len() <= 214 {
                prop_assert!(result.is_ok(), "Failed for valid package name: {}", name);
            } else {
                prop_assert!(result.is_err());
                prop_assert!(result.unwrap_err().contains("too long"));
            }
        }

        #[test]
        fn test_npm_package_name_validation_any(
            name in any_npm_package_name()
        ) {
            let result = validate_npm_package_name(&name);

            // Check various invalid conditions
            if name.is_empty() {
                prop_assert!(result.is_err());
            } else if name.starts_with('.') || name.starts_with('_') {
                prop_assert!(result.is_err());
            } else if name.len() > 214 {
                prop_assert!(result.is_err());
            } else if name.starts_with('@') && !name.contains('/') {
                prop_assert!(result.is_err());
            }
        }

        #[test]
        fn test_docker_image_validation(
            image in docker_image_name()
        ) {
            let result = validate_docker_image_name(&image);

            // Docker image names must be lowercase (but tags can have uppercase)
            let image_part = if let Some(colon_pos) = image.find(':') {
                &image[..colon_pos]
            } else {
                &image
            };

            if image_part.chars().any(|c| c.is_uppercase()) {
                prop_assert!(result.is_err());
            } else if image.contains("://") {
                prop_assert!(result.is_err());
            } else {
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn test_binary_url_validation(
            url in url_string()
        ) {
            let result = validate_binary_url(&url);

            if !url.starts_with("https://") {
                prop_assert!(result.is_err());
                prop_assert!(result.unwrap_err().contains("HTTPS"));
            } else if url.contains("localhost") || url.contains("127.0.0.1") || url.contains("0.0.0.0") {
                prop_assert!(result.is_err());
                prop_assert!(result.unwrap_err().contains("localhost"));
            } else {
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn test_risk_assessment_consistency(
            server_name in prop::string::string_regex("[a-zA-Z0-9@/._:-]+").unwrap()
        ) {
            let risk1 = assess_server_risk_level(&server_name);
            let risk2 = assess_server_risk_level(&server_name);

            // Risk assessment should be deterministic
            prop_assert_eq!(risk1, risk2);
        }

        #[test]
        fn test_server_type_validation_npm(
            package in valid_npm_package_name(),
            version in prop::option::of("[0-9]+\\.[0-9]+\\.[0-9]+"),
        ) {
            let server_type = ServerType::Npm {
                package: package.clone(),
                version: version.clone(),
            };

            let result = validate_server_type_constraints(&server_type);
            if package.len() <= 214 {
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn test_server_type_validation_docker(
            image in docker_image_name(),
            tag in prop::option::of("[a-zA-Z0-9._-]+"),
        ) {
            let server_type = ServerType::Docker {
                image: image.to_lowercase(), // Ensure lowercase for valid test
                tag,
            };

            let result = validate_server_type_constraints(&server_type);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_server_type_validation_binary(
            url in url_string()
        ) {
            let server_type = ServerType::Binary {
                url: url.clone(),
                checksum: None,
            };

            let result = validate_server_type_constraints(&server_type);

            if !url.starts_with("https://") {
                prop_assert!(result.is_err());
            } else if url.contains("localhost") || url.contains("127.0.0.1") {
                prop_assert!(result.is_err());
            } else {
                prop_assert!(result.is_ok());
            }
        }
    }
}
