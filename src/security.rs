use anyhow::{Context, Result};
use url::Url;

/// Security validation for MCP server sources
pub struct SecurityValidator {
    /// List of trusted registries/domains
    trusted_domains: Vec<String>,
    /// Whether to allow HTTP URLs (default: false)
    allow_http: bool,
}

impl SecurityValidator {
    pub fn new() -> Self {
        Self {
            trusted_domains: vec![
                "npmjs.org".to_string(),
                "registry.npmjs.org".to_string(),
                "github.com".to_string(),
                "api.github.com".to_string(),
                "pypi.org".to_string(),
                "hub.docker.com".to_string(),
                "registry.hub.docker.com".to_string(),
            ],
            allow_http: false,
        }
    }

    /// Create a permissive validator that allows HTTP and additional domains
    pub fn permissive() -> Self {
        Self {
            trusted_domains: vec![
                "npmjs.org".to_string(),
                "registry.npmjs.org".to_string(),
                "github.com".to_string(),
                "api.github.com".to_string(),
                "pypi.org".to_string(),
                "hub.docker.com".to_string(),
                "registry.hub.docker.com".to_string(),
                "localhost".to_string(),
                "127.0.0.1".to_string(),
            ],
            allow_http: true,
        }
    }

    /// Add a trusted domain
    pub fn add_trusted_domain(&mut self, domain: &str) {
        if !self.trusted_domains.contains(&domain.to_string()) {
            self.trusted_domains.push(domain.to_string());
        }
    }

    /// Enable/disable HTTP URLs
    pub fn allow_http(&mut self, allow: bool) {
        self.allow_http = allow;
    }

    /// Validate a server source URL
    pub fn validate_url(&self, url_str: &str) -> Result<SecurityValidation> {
        let url = Url::parse(url_str).with_context(|| format!("Invalid URL format: {url_str}"))?;

        let mut validation = SecurityValidation {
            url: url_str.to_string(),
            is_trusted: false,
            is_https: false,
            warnings: Vec::new(),
            domain: None,
        };

        // Check protocol
        match url.scheme() {
            "https" => {
                validation.is_https = true;
            }
            "http" => {
                if !self.allow_http {
                    validation.warnings.push(
                        "HTTP URLs are not secure. Consider using HTTPS if available.".to_string(),
                    );
                }
            }
            scheme => {
                validation.warnings.push(format!(
                    "Unusual URL scheme '{scheme}'. Expected 'https' or 'http'."
                ));
            }
        }

        // Check domain
        if let Some(host) = url.host_str() {
            validation.domain = Some(host.to_string());

            // Check if domain is trusted
            validation.is_trusted = self
                .trusted_domains
                .iter()
                .any(|trusted| host == trusted || host.ends_with(&format!(".{trusted}")));

            if !validation.is_trusted {
                validation.warnings.push(format!(
                    "Domain '{host}' is not in the list of trusted sources. Proceed with caution."
                ));
            }
        }

        Ok(validation)
    }

    /// Validate an NPM package name
    pub fn validate_npm_package(&self, package_name: &str) -> Result<SecurityValidation> {
        let mut validation = SecurityValidation {
            url: format!("npm:{package_name}"),
            is_trusted: true, // NPM packages are generally trusted
            is_https: true,   // NPM registry uses HTTPS
            warnings: Vec::new(),
            domain: Some("npmjs.org".to_string()),
        };

        // Check for suspicious package names
        if package_name.contains("..") || package_name.contains("/..") {
            validation
                .warnings
                .push("Package name contains suspicious path traversal patterns.".to_string());
            validation.is_trusted = false;
        }

        // Check for packages that look like system commands (check this first)
        let suspicious_names = [
            "rm", "del", "sudo", "admin", "root", "system", "kernel", "exec", "eval",
        ];
        if suspicious_names.contains(&package_name) {
            validation.warnings.push(
                "Package name matches system command. Verify this is legitimate.".to_string(),
            );
        }

        // Check for very short or unusual names that might be typosquatting
        if package_name.len() < 3 && !package_name.starts_with('@') && !suspicious_names.contains(&package_name) {
            validation
                .warnings
                .push("Very short package names might be typosquatting attempts.".to_string());
        }

        Ok(validation)
    }

    /// Validate Docker image name
    pub fn validate_docker_image(&self, image_name: &str) -> Result<SecurityValidation> {
        let mut validation = SecurityValidation {
            url: format!("docker:{image_name}"),
            is_trusted: false,
            is_https: true, // Docker Hub uses HTTPS
            warnings: Vec::new(),
            domain: Some("hub.docker.com".to_string()),
        };

        // Split image name into components
        let parts: Vec<&str> = image_name.split('/').collect();

        match parts.len() {
            1 => {
                // Official images (e.g., "nginx", "ubuntu")
                validation.is_trusted = true;
                validation.domain = Some("hub.docker.com".to_string());
            }
            2 => {
                // User/org images (e.g., "user/app")
                let registry = parts[0];
                if self
                    .trusted_domains
                    .iter()
                    .any(|domain| registry == domain || registry.ends_with(&format!(".{domain}")))
                {
                    validation.is_trusted = true;
                }
            }
            3 => {
                // Registry images (e.g., "registry.com/user/app")
                let registry = parts[0];
                validation.domain = Some(registry.to_string());
                validation.is_trusted = self
                    .trusted_domains
                    .iter()
                    .any(|domain| registry == domain || registry.ends_with(&format!(".{domain}")));
            }
            _ => {
                validation
                    .warnings
                    .push("Unusual Docker image format.".to_string());
            }
        }

        // Check for suspicious patterns
        if image_name.contains("..") {
            validation
                .warnings
                .push("Image name contains suspicious path traversal patterns.".to_string());
            validation.is_trusted = false;
        }

        Ok(validation)
    }

    /// Get list of trusted domains
    pub fn trusted_domains(&self) -> &[String] {
        &self.trusted_domains
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of security validation
#[derive(Debug, Clone)]
pub struct SecurityValidation {
    pub url: String,
    pub is_trusted: bool,
    pub is_https: bool,
    pub warnings: Vec<String>,
    pub domain: Option<String>,
}

impl SecurityValidation {
    /// Check if the validation passed without major issues
    pub fn is_safe(&self) -> bool {
        self.is_trusted && self.warnings.is_empty()
    }

    /// Get all warnings as a formatted string
    pub fn warnings_text(&self) -> String {
        if self.warnings.is_empty() {
            return String::new();
        }
        format!("Security warnings:\n{}", self.warnings.join("\n"))
    }

    /// Check if validation should block installation
    pub fn should_block(&self) -> bool {
        // Block if not trusted and has serious warnings
        !self.is_trusted
            && self.warnings.iter().any(|w| {
                w.contains("suspicious") || w.contains("traversal") || w.contains("system command")
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = SecurityValidator::new();
        assert!(!validator.allow_http);
        assert!(validator
            .trusted_domains
            .contains(&"github.com".to_string()));

        let permissive = SecurityValidator::permissive();
        assert!(permissive.allow_http);
        assert!(permissive
            .trusted_domains
            .contains(&"localhost".to_string()));
    }

    #[test]
    fn test_validate_https_url() {
        let validator = SecurityValidator::new();
        let result = validator
            .validate_url("https://github.com/user/repo")
            .unwrap();

        assert!(result.is_trusted);
        assert!(result.is_https);
        assert_eq!(result.domain, Some("github.com".to_string()));
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_http_url() {
        let validator = SecurityValidator::new();
        let result = validator.validate_url("http://example.com/file").unwrap();

        assert!(!result.is_trusted);
        assert!(!result.is_https);
        assert_eq!(result.domain, Some("example.com".to_string()));
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_validate_npm_package() {
        let validator = SecurityValidator::new();

        // Valid package
        let result = validator
            .validate_npm_package("@modelcontextprotocol/server-filesystem")
            .unwrap();
        assert!(result.is_trusted);
        assert!(result.warnings.is_empty());

        // Suspicious package
        let result = validator.validate_npm_package("rm").unwrap();
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("system command"));
    }

    #[test]
    fn test_validate_docker_image() {
        let validator = SecurityValidator::new();

        // Official image
        let result = validator.validate_docker_image("nginx").unwrap();
        assert!(result.is_trusted);

        // User image
        let result = validator.validate_docker_image("user/app").unwrap();
        assert!(!result.is_trusted); // Not in trusted domains

        // Registry image
        let result = validator
            .validate_docker_image("github.com/user/app")
            .unwrap();
        assert!(result.is_trusted); // github.com is trusted
    }

    #[test]
    fn test_add_trusted_domain() {
        let mut validator = SecurityValidator::new();
        validator.add_trusted_domain("example.com");

        let result = validator
            .validate_url("https://example.com/package")
            .unwrap();
        assert!(result.is_trusted);
    }

    #[test]
    fn test_security_validation_methods() {
        let validation = SecurityValidation {
            url: "https://example.com".to_string(),
            is_trusted: true,
            is_https: true,
            warnings: vec!["Test warning".to_string()],
            domain: Some("example.com".to_string()),
        };

        assert!(!validation.is_safe()); // Has warnings
        assert!(!validation.warnings_text().is_empty());
        assert!(!validation.should_block()); // Trusted with non-serious warnings
    }
}
