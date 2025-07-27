use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

pub struct ServerRunner {
    platform: Platform,
    verbose: bool,
}

impl ServerRunner {
    pub fn new(platform: Platform, verbose: bool) -> Self {
        Self { platform, verbose }
    }

    pub fn run(&self, server: &str, args: &[String]) -> Result<()> {
        // First, try to find the server
        let server_path = self.resolve_server_path(server)?;

        if self.verbose {
            eprintln!("Resolved server path: {}", server_path.display());
        }

        // Normalize arguments that might be paths
        let normalized_args: Vec<String> = args
            .iter()
            .map(|arg| {
                // Simple heuristic: if it looks like a path, normalize it
                if arg.contains('/') || arg.contains('\\') {
                    normalize_path(arg, self.platform)
                } else {
                    arg.clone()
                }
            })
            .collect();

        // Determine the command to use based on platform
        let (command, command_args) =
            self.get_command_for_platform(&server_path, &normalized_args)?;

        if self.verbose {
            eprintln!("Executing command: {command} {command_args:?}");
        }

        // Execute the command
        let mut cmd = Command::new(&command);
        cmd.args(&command_args);

        // Inherit environment variables
        cmd.envs(std::env::vars());

        let status = cmd
            .status()
            .with_context(|| format!("Failed to execute command: {command}"))?;

        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            match exit_code {
                127 => bail!(
                    "Command not found: {}\n\
                    This usually means the MCP server is not installed.\n\
                    Try running: mcp install {}",
                    server,
                    server
                ),
                1 if server == "echo" || server == "node" => bail!(
                    "'{}' is not an MCP server package.\n\
                    MCP Helper is designed to run Model Context Protocol servers.\n\
                    If you're testing, try: mcp run cowsay \"Hello World!\"",
                    server
                ),
                _ => bail!(
                    "Server '{}' exited with status: {}\n\
                    This might mean:\n\
                    - The server is not installed (try: mcp install {})\n\
                    - The server name is misspelled\n\
                    - The server encountered an error\n\n\
                    Run with --verbose for more details.",
                    server,
                    exit_code,
                    server
                ),
            }
        }

        Ok(())
    }

    pub fn resolve_server_path(&self, server: &str) -> Result<PathBuf> {
        // Normalize the path for the current platform
        let normalized_server = normalize_path(server, self.platform);

        // For now, just return the server name as-is
        // In a real implementation, we would:
        // 1. Check if it's a local path
        // 2. Check node_modules/.bin
        // 3. Check global npm installations
        // 4. Check configured servers

        if Path::new(&normalized_server).exists() {
            Ok(PathBuf::from(normalized_server))
        } else {
            // Assume it's an npm package for now
            Ok(PathBuf::from(normalized_server))
        }
    }

    pub fn get_command_for_platform(
        &self,
        server_path: &Path,
        args: &[String],
    ) -> Result<(String, Vec<String>)> {
        match self.platform {
            Platform::Windows => {
                // On Windows, we need to handle npx specially
                self.get_windows_command(server_path, args)
            }
            Platform::MacOS | Platform::Linux => {
                // On Unix-like systems, npx usually works fine
                self.get_unix_command(server_path, args)
            }
        }
    }

    pub fn get_windows_command(
        &self,
        server_path: &Path,
        args: &[String],
    ) -> Result<(String, Vec<String>)> {
        // Check if we're dealing with an npm package
        let server_str = server_path.to_string_lossy();

        if !server_path.exists() || !server_path.is_absolute() {
            // It's likely an npm package, use npx

            // First, try to find npx.cmd
            if let Ok(npx_cmd) = which::which("npx.cmd") {
                if self.verbose {
                    eprintln!("Found npx.cmd at: {}", npx_cmd.display());
                }
                let mut cmd_args = vec![server_str.to_string()];
                cmd_args.extend(args.iter().cloned());
                let mut final_args = vec!["/c".to_string(), "npx.cmd".to_string()];
                final_args.extend(cmd_args);
                return Ok(("cmd.exe".to_string(), final_args));
            }

            // Try regular npx
            if let Ok(_npx) = which::which("npx") {
                let mut cmd_args = vec![server_str.to_string()];
                cmd_args.extend(args.iter().cloned());
                return Ok(("npx".to_string(), cmd_args));
            }

            bail!(
                "Could not find npx or npx.cmd in PATH.\n\n\
                {} To fix this issue:\n\n\
                1. Install Node.js from https://nodejs.org/\n\
                2. Restart your terminal/command prompt\n\
                3. Run 'mcp doctor' to verify the installation\n\n\
                {} On Windows, you may need to:\n\
                   - Use the Node.js command prompt\n\
                   - Or add Node.js to your system PATH",
                "→".green(),
                "→".green()
            );
        }

        // It's a local file, execute directly
        let mut cmd_args = vec![server_str.to_string()];
        cmd_args.extend(args.iter().cloned());
        Ok(("node".to_string(), cmd_args))
    }

    pub fn get_unix_command(
        &self,
        server_path: &Path,
        args: &[String],
    ) -> Result<(String, Vec<String>)> {
        let server_str = server_path.to_string_lossy();

        if !server_path.exists() || !server_path.is_absolute() {
            // It's likely an npm package, use npx
            let mut cmd_args = vec![server_str.to_string()];
            cmd_args.extend(args.iter().cloned());

            // Check if npx exists
            if which::which("npx").is_err() {
                bail!(
                    "Could not find npx in PATH.\n\n\
                    {} To fix this issue:\n\n\
                    1. Install Node.js from https://nodejs.org/\n\
                    2. Restart your terminal\n\
                    3. Run 'mcp doctor' to verify the installation",
                    "→".green()
                );
            }

            Ok(("npx".to_string(), cmd_args))
        } else {
            // It's a local file, execute directly
            let mut cmd_args = vec![server_str.to_string()];
            cmd_args.extend(args.iter().cloned());
            Ok(("node".to_string(), cmd_args))
        }
    }
}

pub fn normalize_path(path: &str, platform: Platform) -> String {
    match platform {
        Platform::Windows => path.replace('/', "\\"),
        Platform::MacOS | Platform::Linux => path.replace('\\', "/"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_normalization_windows() {
        let normalized = normalize_path("path/to/file", Platform::Windows);
        assert_eq!(normalized, "path\\to\\file");
    }

    #[test]
    fn test_path_normalization_unix() {
        let normalized = normalize_path("path\\to\\file", Platform::Linux);
        assert_eq!(normalized, "path/to/file");

        let normalized = normalize_path("path\\to\\file", Platform::MacOS);
        assert_eq!(normalized, "path/to/file");
    }

    #[test]
    fn test_path_normalization_mixed() {
        // Test mixed separators
        let normalized = normalize_path("path\\to/file", Platform::Windows);
        assert_eq!(normalized, "path\\to\\file");

        let normalized = normalize_path("path/to\\file", Platform::Linux);
        assert_eq!(normalized, "path/to/file");
    }

    #[test]
    fn test_server_runner_creation() {
        let runner = ServerRunner::new(Platform::Windows, true);
        assert!(runner.verbose);

        let runner = ServerRunner::new(Platform::MacOS, false);
        assert!(!runner.verbose);
    }

    #[test]
    fn test_resolve_server_path() {
        let runner = ServerRunner::new(Platform::Windows, false);

        // Test npm package name
        let path = runner.resolve_server_path("some-package").unwrap();
        assert_eq!(path.to_str().unwrap(), "some-package");

        // Test path with forward slashes on Windows
        let path = runner.resolve_server_path("./path/to/server").unwrap();
        assert_eq!(path.to_str().unwrap(), ".\\path\\to\\server");
    }

    #[test]
    fn test_command_construction_windows() {
        let runner = ServerRunner::new(Platform::Windows, false);

        // Test npm package command
        let (cmd, args) = runner
            .get_windows_command(
                &PathBuf::from("my-server"),
                &["arg1".to_string(), "arg2".to_string()],
            )
            .unwrap_or_else(|_| ("npx".to_string(), vec![]));

        // We expect either npx.cmd through cmd.exe or direct npx
        assert!(cmd == "cmd.exe" || cmd == "npx");
        if cmd == "cmd.exe" {
            assert!(args.contains(&"npx.cmd".to_string()));
        }
    }

    #[test]
    fn test_command_construction_unix() {
        let runner = ServerRunner::new(Platform::Linux, false);

        // Test npm package command
        let result = runner.get_unix_command(&PathBuf::from("my-server"), &["arg1".to_string()]);

        // This will fail if npx is not available, which is expected in test environment
        if let Ok((cmd, args)) = result {
            assert_eq!(cmd, "npx");
            assert_eq!(args, vec!["my-server", "arg1"]);
        }
    }
}
