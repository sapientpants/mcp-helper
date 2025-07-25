use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::env;

mod runner;

// Import Platform from runner module
use runner::Platform;

// Import from mcp_helper lib
use mcp_helper::install::InstallCommand;

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("{}", "Verbose mode enabled".dimmed());
    }

    match cli.command {
        Commands::Run { server, args } => {
            run_server(&server, &args, cli.verbose)?;
        }
        Commands::Install { server } => {
            println!("{} Installing MCP server: {}", "→".green(), server.cyan());
            let install = InstallCommand::new(cli.verbose);
            install.execute(&server)?;
        }
        Commands::Setup => {
            println!("{}", "🔧 Running MCP Helper setup...".blue().bold());
            println!("{}", "Setup command not yet implemented".yellow());
        }
        Commands::Config { action } => match action {
            ConfigAction::Add { server } => {
                println!("{} Adding server to config: {}", "→".green(), server.cyan());
                println!("{}", "Config add command not yet implemented".yellow());
            }
            ConfigAction::List => {
                println!("{}", "📋 Configured MCP servers:".blue().bold());
                println!("{}", "Config list command not yet implemented".yellow());
            }
            ConfigAction::Remove { server } => {
                println!(
                    "{} Removing server from config: {}",
                    "→".green(),
                    server.cyan()
                );
                println!("{}", "Config remove command not yet implemented".yellow());
            }
        },
        Commands::Doctor => {
            println!("{}", "🏥 Running MCP diagnostics...".blue().bold());
            println!("{}", "Doctor command not yet implemented".yellow());
        }
    }

    Ok(())
}

fn run_server(server: &str, args: &[String], verbose: bool) -> Result<()> {
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
            Platform::Windows | Platform::MacOS | Platform::Linux => assert!(true),
        }
    }
}
