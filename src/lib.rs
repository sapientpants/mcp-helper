pub mod runner;

// Re-export Platform enum so it can be used in tests
pub use runner::Platform;

#[cfg(test)]
mod integration_tests {
    use std::process::Command;

    #[test]
    fn test_cli_help() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("MCP Helper - Make MCP Just Work™"));
        assert!(stdout.contains("run"));
        assert!(stdout.contains("install"));
        assert!(stdout.contains("setup"));
        assert!(stdout.contains("config"));
        assert!(stdout.contains("doctor"));
    }

    #[test]
    fn test_cli_version() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--version"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // The binary name in clap is "mcp", not "mcp-helper"
        assert!(stdout.contains("mcp") || stdout.contains("mcp-helper"));
        assert!(stdout.contains("0.1.0"));
    }

    #[test]
    fn test_run_command_help() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "run", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Run an MCP server"));
        assert!(stdout.contains("Name of the MCP server to run"));
    }

    #[test]
    fn test_config_subcommands() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "config", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("add"));
        assert!(stdout.contains("list"));
        assert!(stdout.contains("remove"));
    }
}
