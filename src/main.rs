use clap::{Parser, Subcommand};
use colored::Colorize;

// Import from mcp_helper lib
use mcp_helper::error::McpError;
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
    #[command(about = "Add an MCP server to client configuration")]
    Add {
        #[arg(help = "Name or path of the MCP server to add")]
        server: String,

        #[arg(long, help = "Command to execute (e.g., npx, python, docker)")]
        command: Option<String>,

        #[arg(long, help = "Arguments for the command")]
        args: Vec<String>,

        #[arg(long, help = "Environment variables in KEY=VALUE format")]
        env: Vec<String>,

        #[arg(long, help = "Skip interactive prompts")]
        non_interactive: bool,
    },

    #[command(about = "List configured MCP servers")]
    List {
        #[arg(short, long, help = "Show detailed information")]
        verbose: bool,
    },

    #[command(about = "Remove an MCP server from configuration")]
    Remove {
        #[arg(help = "Name of the server to remove")]
        server: String,

        #[arg(long, help = "Remove from all clients")]
        all: bool,
    },

    #[command(about = "Install an MCP server", hide = true)] // Hidden/deprecated
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

    #[command(about = "Quick environment check (first-time setup)")]
    Setup,

    #[command(about = "Manage MCP server configurations", hide = true)] // Hidden/deprecated
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(about = "Comprehensive diagnostics (troubleshooting)")]
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

fn main() {
    let cli = Cli::parse();

    setup_logging(&cli);

    let result = execute_command(cli);

    handle_result(result);
}

/// Set up logging based on CLI arguments
fn setup_logging(cli: &Cli) {
    if let Err(e) = logging::init_logging(cli.verbose) {
        eprintln!("Warning: Failed to initialize logging: {e}");
    }

    if cli.verbose {
        logging::log_system_info();
        eprintln!("{}", "Verbose mode enabled".dimmed());
    }
}

/// Execute the requested command
fn execute_command(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Add {
            server,
            command,
            args,
            env,
            non_interactive,
        } => execute_add_command(server, command, args, env, non_interactive, cli.verbose),
        Commands::List { verbose } => execute_list_command(verbose || cli.verbose),
        Commands::Remove { server, all } => execute_remove_command(server, all, cli.verbose),
        Commands::Install {
            server,
            auto_install_deps,
            dry_run,
            config,
            batch,
        } => execute_install_command(
            server,
            auto_install_deps,
            dry_run,
            config,
            batch,
            cli.verbose,
        ),
        Commands::Setup => execute_setup_command(),
        Commands::Config { action } => execute_config_command(action),
        Commands::Doctor => execute_doctor_command(),
    }
}

/// Execute the install command (deprecated - redirects to add)
fn execute_install_command(
    server: String,
    _auto_install_deps: bool,
    _dry_run: bool,
    config: Vec<String>,
    batch: Option<String>,
    verbose: bool,
) -> anyhow::Result<()> {
    eprintln!(
        "{} The 'install' command is deprecated. Please use 'mcp add' instead.",
        "⚠".yellow()
    );

    if batch.is_some() {
        eprintln!("Batch installation is not yet supported in 'mcp add'.");
        return Err(anyhow::anyhow!("Batch mode not supported"));
    }

    // Parse config overrides into env vars
    let env: Vec<String> = config;

    // Redirect to add command
    execute_add_command(server, None, Vec::new(), env, false, verbose)
}

/// Execute the setup command
fn execute_setup_command() -> anyhow::Result<()> {
    use mcp_helper::setup::SetupCommand;

    let setup = SetupCommand::new(false); // verbose is global, not passed here
    setup.execute().map_err(convert_mcp_error)
}

/// Execute the add command
fn execute_add_command(
    server: String,
    command: Option<String>,
    args: Vec<String>,
    env: Vec<String>,
    non_interactive: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    use mcp_helper::add::AddCommand;

    let mut cmd = AddCommand::new(verbose);

    // Parse environment variables
    let mut env_map = std::collections::HashMap::new();
    for env_var in env {
        if let Some((key, value)) = env_var.split_once('=') {
            env_map.insert(key.to_string(), value.to_string());
        }
    }

    cmd.execute(&server, command, args, env_map, non_interactive)
        .map_err(convert_mcp_error)
}

/// Execute the list command
fn execute_list_command(verbose: bool) -> anyhow::Result<()> {
    use mcp_helper::config_commands::ConfigListCommand;

    let cmd = ConfigListCommand::new(verbose);
    cmd.execute().map_err(convert_mcp_error)
}

/// Execute the remove command
fn execute_remove_command(server: String, all: bool, verbose: bool) -> anyhow::Result<()> {
    use mcp_helper::config_commands::ConfigRemoveCommand;

    let mut cmd = ConfigRemoveCommand::new(verbose);
    cmd.set_remove_all(all);
    cmd.execute(&server).map_err(convert_mcp_error)
}

/// Execute config commands (deprecated - redirects to new top-level commands)
fn execute_config_command(action: ConfigAction) -> anyhow::Result<()> {
    eprintln!(
        "{} The 'config' subcommands are deprecated. Please use top-level commands instead:",
        "⚠".yellow()
    );
    eprintln!("  • 'mcp list' instead of 'mcp config list'");
    eprintln!("  • 'mcp add' instead of 'mcp config add'");
    eprintln!("  • 'mcp remove' instead of 'mcp config remove'");
    eprintln!();

    match action {
        ConfigAction::Add { server } => {
            execute_add_command(server, None, Vec::new(), Vec::new(), false, false)
        }
        ConfigAction::List => execute_list_command(false),
        ConfigAction::Remove { server } => execute_remove_command(server, false, false),
    }
}

/// Execute the doctor command
fn execute_doctor_command() -> anyhow::Result<()> {
    use mcp_helper::doctor::DoctorCommand;

    let doctor = DoctorCommand::new(false); // verbose is global, not passed here
    doctor.execute().map_err(convert_mcp_error)
}

/// Convert McpError to anyhow::Error
fn convert_mcp_error(e: McpError) -> anyhow::Error {
    match e {
        McpError::Other(err) => err,
        _ => anyhow::anyhow!("{}", e),
    }
}

/// Handle the result of command execution
fn handle_result(result: anyhow::Result<()>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Just ensure the CLI can be created
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
