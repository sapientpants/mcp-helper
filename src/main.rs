use clap::{Parser, Subcommand};
use colored::*;
use std::env;

mod runner;

// Import Platform from runner module
use runner::Platform;

// Import from mcp_helper lib
use mcp_helper::error::McpError;
use mcp_helper::install::InstallCommand;
use mcp_helper::logging;

#[derive(Parser)]
#[command(name = "mcp")]
#[command(author = "MCP Helper Contributors")]
#[command(version = "0.1.0")]
#[command(about = "MCP Helper - Make MCP Just Work™", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, help = "Enable verbose output", global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Run an MCP server")]
    Run {
        #[arg(help = "Name of the MCP server to run")]
        server: String,

        #[arg(
            trailing_var_arg = true,
            help = "Additional arguments to pass to the server"
        )]
        args: Vec<String>,
    },

    #[command(about = "Install an MCP server")]
    Install {
        #[arg(help = "Name or path of the MCP server to install")]
        server: String,

        #[arg(long, help = "Automatically install missing dependencies")]
        auto_install_deps: bool,

        #[arg(long, help = "Show what would be done without making changes")]
        dry_run: bool,

        #[arg(long, help = "Configuration in key=value format (skips prompts)")]
        config: Vec<String>,

        #[arg(long, help = "Install servers from batch file")]
        batch: Option<String>,
    },

    #[command(about = "One-time setup for your OS")]
    Setup,

    #[command(about = "Manage MCP server configurations")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(about = "Diagnose and fix common MCP issues")]
    Doctor,
}

#[derive(Subcommand)]
enum ConfigAction {
    #[command(about = "Add a server to configuration")]
    Add {
        #[arg(help = "Name of the server")]
        server: String,
    },
    #[command(about = "List all configured servers")]
    List,
    #[command(about = "Remove a server from configuration")]
    Remove {
        #[arg(help = "Name of the server")]
        server: String,
    },
}

fn print_not_implemented(command: &str) {
    println!(
        "{}",
        format!("{command} command not yet implemented").yellow()
    );
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    if let Err(e) = logging::init_logging(cli.verbose) {
        eprintln!("Warning: Failed to initialize logging: {e}");
    }

    // Log system information in verbose mode
    if cli.verbose {
        logging::log_system_info();
        eprintln!("{}", "Verbose mode enabled".dimmed());
    }

    let result = match cli.command {
        Commands::Run { server, args } => run_server(&server, &args, cli.verbose),
        Commands::Install {
            server,
            auto_install_deps,
            dry_run,
            config,
            batch,
        } => {
            let mut install = InstallCommand::new(cli.verbose);
            install = install
                .with_auto_install_deps(auto_install_deps)
                .with_dry_run(dry_run)
                .with_config_overrides(config);

            if let Some(batch_file) = batch {
                println!(
                    "{} Installing servers from batch file: {}",
                    "→".green(),
                    batch_file.cyan()
                );
                install.execute_batch(&batch_file).map_err(|e| match e {
                    McpError::Other(err) => err,
                    _ => anyhow::anyhow!("{}", e),
                })
            } else {
                println!("{} Installing MCP server: {}", "→".green(), server.cyan());
                install.execute(&server).map_err(|e| match e {
                    McpError::Other(err) => err,
                    _ => anyhow::anyhow!("{}", e),
                })
            }
        }
        Commands::Setup => {
            println!("{}", "🔧 Running MCP Helper setup...".blue().bold());
            print_not_implemented("Setup");
            Ok(())
        }
        Commands::Config { action } => {
            match action {
                ConfigAction::Add { server } => {
                    println!("{} Adding server to config: {}", "→".green(), server.cyan());
                    print_not_implemented("Config add");
                }
                ConfigAction::List => {
                    println!("{}", "📋 Configured MCP servers:".blue().bold());
                    print_not_implemented("Config list");
                }
                ConfigAction::Remove { server } => {
                    println!(
                        "{} Removing server from config: {}",
                        "→".green(),
                        server.cyan()
                    );
                    print_not_implemented("Config remove");
                }
            }
            Ok(())
        }
        Commands::Doctor => {
            println!("{}", "🏥 Running MCP diagnostics...".blue().bold());
            print_not_implemented("Doctor");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!();
        match e.downcast::<McpError>() {
            Ok(mcp_err) => {
                eprintln!("{mcp_err}");
            }
            Err(err) => {
                eprintln!("{} {}", "✗".red().bold(), err);
            }
        }
        std::process::exit(1);
    }
}

fn run_server(server: &str, args: &[String], verbose: bool) -> anyhow::Result<()> {
    println!(
        "{} Running MCP server: {}",
        "🚀".green(),
        server.cyan().bold()
    );

    // Detect platform
    let platform = detect_platform();
    if verbose {
        eprintln!("{} Detected platform: {:?}", "ℹ".blue(), platform);
    }

    // Create and use the server runner
    let runner = runner::ServerRunner::new(platform, verbose);
    runner.run(server, args)?;

    Ok(())
}

fn detect_platform() -> Platform {
    match env::consts::OS {
        "windows" => Platform::Windows,
        "macos" => Platform::MacOS,
        "linux" => Platform::Linux,
        _ => {
            eprintln!(
                "{} Unknown platform: {}, defaulting to Linux behavior",
                "⚠".yellow(),
                env::consts::OS
            );
            Platform::Linux
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = detect_platform();
        // Just ensure it returns something valid
        match platform {
            Platform::Windows | Platform::MacOS | Platform::Linux => {}
        }
    }

    #[test]
    fn test_print_not_implemented() {
        // Test that the function runs without panicking
        print_not_implemented("TestCommand");
    }

    #[test]
    fn test_run_server_error_handling() {
        // Test run_server with invalid server name
        let result = run_server("nonexistent-server-xyz", &[], false);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_server_verbose() {
        // Test run_server with verbose mode
        let result = run_server("nonexistent-server-xyz", &["arg1".to_string()], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_platform_detection_current_os() {
        let platform = detect_platform();
        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows);
        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);
        #[cfg(target_os = "linux")]
        assert_eq!(platform, Platform::Linux);
    }
}
