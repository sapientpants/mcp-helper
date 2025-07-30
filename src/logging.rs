use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize structured logging based on verbosity level
pub fn init_logging(verbose: bool) -> Result<()> {
    let env_filter = if verbose {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("mcp_helper=debug,info"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("mcp_helper=info,warn,error"))
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_ansi(true)
        .compact();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;

    if verbose {
        tracing::info!("Verbose logging enabled");
    }

    Ok(())
}

/// Log dependency check operations
pub fn log_dependency_check(dependency: &str, status: &str) {
    tracing::info!(
        dependency = dependency,
        status = status,
        "Dependency check completed"
    );
}

/// Log configuration changes
pub fn log_config_change(client: &str, server: &str, action: &str) {
    tracing::info!(
        client = client,
        server = server,
        action = action,
        "Configuration change"
    );
}

/// Log HTTP requests
pub fn log_http_request(method: &str, url: &str, status: Option<u16>) {
    if let Some(status_code) = status {
        tracing::info!(
            method = method,
            url = url,
            status = status_code,
            "HTTP request completed"
        );
    } else {
        tracing::debug!(method = method, url = url, "HTTP request initiated");
    }
}

/// Log system information for debugging
pub fn log_system_info() {
    tracing::debug!(
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        "System information"
    );
}

/// Log server installation events
pub fn log_server_installation(server: &str, server_type: &str, success: bool) {
    if success {
        tracing::info!(
            server = server,
            server_type = server_type,
            "Server installation successful"
        );
    } else {
        tracing::error!(
            server = server,
            server_type = server_type,
            "Server installation failed"
        );
    }
}

/// Log performance metrics
pub fn log_performance(operation: &str, duration_ms: u64) {
    tracing::debug!(
        operation = operation,
        duration_ms = duration_ms,
        "Operation performance"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_verbose() {
        // This test ensures the function doesn't panic
        // We can't easily test the actual logging setup without more complex machinery
        let result = init_logging(true);
        // It might fail if already initialized, which is ok
        let _ = result;
    }

    #[test]
    fn test_init_logging_normal() {
        let result = init_logging(false);
        // It might fail if already initialized, which is ok
        let _ = result;
    }

    #[test]
    fn test_logging_functions() {
        // Test that logging functions don't panic
        log_dependency_check("node", "installed");
        log_config_change("claude", "test-server", "add");
        log_http_request("GET", "https://example.com", Some(200));
        log_http_request("POST", "https://example.com", None);
        log_system_info();
        log_server_installation("test-server", "npm", true);
        log_server_installation("test-server", "npm", false);
        log_performance("dependency_check", 150);
    }
}
